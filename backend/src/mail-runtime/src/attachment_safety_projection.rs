//! Event-only projection of canonical Communications attachment safety state.

use makosh_communications_attachment_contract::{
    admission::communication_attachment_safety_state_changed_contract_reference_v1,
    lifecycle_v1::{AttachmentSafetyStateChangedV1, AttachmentSafetyStateV1},
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_persistence::{
    MailAttachmentSafetyStateV1 as PersistedSafetyStateV1, MailAttachmentSafetyTransitionV1,
    MailDurablePersistence, MailDurablePersistenceError,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

const COMMUNICATIONS_RUNTIME_MODULE_ID: &str = "communications-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAttachmentSafetyProjectionErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailDurablePersistenceError),
}

pub async fn project_attachment_safety_state_changed_v1(
    durable: &MailDurablePersistence,
    exact_event_bytes: &[u8],
    consumed_at_unix_seconds: i64,
) -> Result<bool, MailAttachmentSafetyProjectionErrorV1> {
    let event_record = OutboxRecordV1::accept(exact_event_bytes.to_vec())
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(event_record.exact_bytes())
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope)?;
    let transition = decode_transition(&envelope)?;
    durable
        .apply_attachment_safety_transition(&event_record, transition, consumed_at_unix_seconds)
        .await
        .map_err(MailAttachmentSafetyProjectionErrorV1::Persistence)
}

pub async fn consume_next_attachment_safety_state_changed_v1(
    durable: &MailDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    consumed_at_unix_seconds: i64,
) -> Result<bool, MailAttachmentSafetyProjectionErrorV1> {
    if !exact_permit_contract(
        permit.contract(),
        &communication_attachment_safety_state_changed_contract_reference_v1(),
    ) {
        return Err(MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope);
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::Unavailable)?;
    let applied = project_attachment_safety_state_changed_v1(
        durable,
        delivery.exact_bytes(),
        consumed_at_unix_seconds,
    )
    .await?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::Unavailable)?;
    Ok(applied)
}

fn decode_transition(
    envelope: &DurableEnvelopeV1,
) -> Result<MailAttachmentSafetyTransitionV1, MailAttachmentSafetyProjectionErrorV1> {
    if !exact_contract(
        envelope.contract.as_ref(),
        &communication_attachment_safety_state_changed_contract_reference_v1(),
    ) || !matches!(envelope.semantics, Some(Semantics::Event(_)))
        || envelope.source.as_ref().is_none_or(|source| {
            source.module_id != COMMUNICATIONS_RUNTIME_MODULE_ID || source.runtime_generation == 0
        })
    {
        return Err(MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope);
    }
    let payload = AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::InvalidPayload)?;
    let attachment_anchor_id = id16(&payload.attachment_anchor_id)?;
    let evidence_id = id16(&payload.evidence_id)?;
    if envelope.partition_key.as_slice() != attachment_anchor_id.as_slice()
        || id16(&envelope.causation_message_id).is_err()
        || id16(&envelope.message_id).is_err()
        || id16(&envelope.correlation_id).is_err()
    {
        return Err(MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope);
    }
    Ok(MailAttachmentSafetyTransitionV1 {
        attachment_anchor_id,
        expected_state: safety_state(payload.expected_state)?,
        next_state: safety_state(payload.next_state)?,
        evidence_id,
        observed_at_unix_seconds: payload.observed_at_unix_seconds,
    })
}

fn safety_state(
    value: i32,
) -> Result<PersistedSafetyStateV1, MailAttachmentSafetyProjectionErrorV1> {
    match AttachmentSafetyStateV1::try_from(value)
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::InvalidPayload)?
    {
        AttachmentSafetyStateV1::DescriptorOnly => Ok(PersistedSafetyStateV1::DescriptorOnly),
        AttachmentSafetyStateV1::BlobPending => Ok(PersistedSafetyStateV1::BlobPending),
        AttachmentSafetyStateV1::BlobAdmitted => Ok(PersistedSafetyStateV1::BlobAdmitted),
        AttachmentSafetyStateV1::Quarantined => Ok(PersistedSafetyStateV1::Quarantined),
        AttachmentSafetyStateV1::SafeForDelivery => Ok(PersistedSafetyStateV1::SafeForDelivery),
        AttachmentSafetyStateV1::Rejected => Ok(PersistedSafetyStateV1::Rejected),
        AttachmentSafetyStateV1::Unspecified => {
            Err(MailAttachmentSafetyProjectionErrorV1::InvalidPayload)
        }
    }
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

fn id16(value: &[u8]) -> Result<[u8; 16], MailAttachmentSafetyProjectionErrorV1> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| MailAttachmentSafetyProjectionErrorV1::InvalidPayload)?;
    (!id.iter().all(|byte| *byte == 0))
        .then_some(id)
        .ok_or(MailAttachmentSafetyProjectionErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::{EventMetadataV1, SourceRefV1};

    #[test]
    fn canonical_event_requires_exact_source_partition_and_nonzero_lineage() {
        let payload = AttachmentSafetyStateChangedV1 {
            attachment_anchor_id: vec![1; 16],
            expected_state: AttachmentSafetyStateV1::BlobAdmitted as i32,
            next_state: AttachmentSafetyStateV1::SafeForDelivery as i32,
            evidence_id: vec![2; 16],
            observed_at_unix_seconds: 1_700_000_000,
        };
        let mut envelope = DurableEnvelopeV1 {
            message_id: vec![3; 16],
            causation_message_id: vec![6; 16],
            correlation_id: vec![4; 16],
            partition_key: vec![1; 16],
            contract: Some(wire_contract(
                communication_attachment_safety_state_changed_contract_reference_v1(),
            )),
            source: Some(SourceRefV1 {
                module_id: COMMUNICATIONS_RUNTIME_MODULE_ID.to_owned(),
                runtime_instance_id: vec![5; 16],
                runtime_generation: 1,
            }),
            semantics: Some(Semantics::Event(EventMetadataV1 { occurred_at: None })),
            payload: payload.encode_to_vec(),
            ..DurableEnvelopeV1::default()
        };
        assert!(decode_transition(&envelope).is_ok());

        envelope.source.as_mut().expect("source").module_id =
            "attachment-security-runtime".to_owned();
        assert_eq!(
            decode_transition(&envelope),
            Err(MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope)
        );

        envelope.source.as_mut().expect("source").module_id =
            COMMUNICATIONS_RUNTIME_MODULE_ID.to_owned();
        envelope.causation_message_id.clear();
        assert_eq!(
            decode_transition(&envelope),
            Err(MailAttachmentSafetyProjectionErrorV1::InvalidEnvelope)
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
