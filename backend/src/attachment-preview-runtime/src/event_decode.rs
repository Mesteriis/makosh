use makosh_attachment_preview_core::{
    AttachmentPreviewSafetyFactV1, AttachmentPreviewSafetyStateV1,
    AttachmentPreviewScanCandidateFactV1,
};
use makosh_attachment_preview_ingress::{
    attachment_preview_custody_delegated_contract_reference_v1,
    attachment_preview_custody_delegation_rejected_contract_reference_v1,
    wire::{AttachmentPreviewCustodyDelegatedV1, AttachmentPreviewCustodyDelegationRejectedV1},
};
use makosh_attachment_security_contract::{
    admission::attachment_security_scan_candidate_observed_contract_reference_v1,
    v1::AttachmentSecurityScanCandidateObservedV1,
};
use makosh_communications_attachment_contract::{
    admission::communication_attachment_safety_state_changed_contract_reference_v1,
    lifecycle_v1::{AttachmentSafetyStateChangedV1, AttachmentSafetyStateV1},
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

pub(crate) struct DecodedCandidateV1 {
    pub fact: AttachmentPreviewScanCandidateFactV1,
    pub payload_sha256: [u8; 32],
}

pub(crate) struct DecodedSafetyV1 {
    pub fact: AttachmentPreviewSafetyFactV1,
    pub envelope_sha256: [u8; 32],
    pub payload_sha256: [u8; 32],
}

pub(crate) enum DecodedCustodyResultV1 {
    Delegated {
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: AttachmentPreviewCustodyDelegatedV1,
    },
    Rejected {
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: AttachmentPreviewCustodyDelegationRejectedV1,
    },
}

pub(crate) fn decode_candidate_v1(
    exact_bytes: &[u8],
) -> Result<DecodedCandidateV1, DeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_bytes)?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(DeliveryDecodeErrorV1::WrongContract);
    };
    let payload = AttachmentSecurityScanCandidateObservedV1::decode(envelope.payload.as_slice())
        .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &attachment_security_scan_candidate_observed_contract_reference_v1(),
    ) || metadata.observation_id != envelope.message_id
        || envelope.partition_key != payload.attachment_anchor_id
        || payload.declared_size == 0
        || !(1..=2_048).contains(&payload.custody_transfer_source_proof.len())
    {
        return Err(DeliveryDecodeErrorV1::InvalidPayload);
    }
    Ok(DecodedCandidateV1 {
        fact: AttachmentPreviewScanCandidateFactV1 {
            attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
            candidate_message_id: id16(&envelope.message_id)?,
            candidate_envelope_sha256: envelope_sha256,
            source_reference_id: id16(&payload.blob_reference_id)?,
            declared_size: payload.declared_size,
            source_receipt_sha256: id32(&payload.blob_receipt_sha256)?,
            custody_transfer_source_proof: payload.custody_transfer_source_proof,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        payload_sha256: Sha256::digest(&envelope.payload).into(),
    })
}

pub(crate) fn decode_safety_v1(
    exact_bytes: &[u8],
) -> Result<Option<DecodedSafetyV1>, DeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_bytes)?;
    let Some(Semantics::Event(_)) = envelope.semantics.as_ref() else {
        return Err(DeliveryDecodeErrorV1::WrongContract);
    };
    let payload = AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice())
        .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &communication_attachment_safety_state_changed_contract_reference_v1(),
    ) || envelope.partition_key != payload.attachment_anchor_id
    {
        return Err(DeliveryDecodeErrorV1::InvalidPayload);
    }
    let next_state = safety_state(payload.next_state)?;
    if !matches!(
        next_state,
        AttachmentPreviewSafetyStateV1::SafeForDelivery
            | AttachmentPreviewSafetyStateV1::Quarantined
            | AttachmentPreviewSafetyStateV1::Rejected
    ) {
        return Ok(None);
    }
    Ok(Some(DecodedSafetyV1 {
        fact: AttachmentPreviewSafetyFactV1 {
            attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
            safety_message_id: id16(&envelope.message_id)?,
            safety_evidence_id: id16(&payload.evidence_id)?,
            expected_state: safety_state(payload.expected_state)?,
            next_state,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        envelope_sha256,
        payload_sha256: Sha256::digest(&envelope.payload).into(),
    }))
}

pub(crate) fn decode_custody_result_v1(
    exact_bytes: &[u8],
) -> Result<DecodedCustodyResultV1, DeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_bytes)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(DeliveryDecodeErrorV1::WrongContract);
    };
    let message_id = id16(&envelope.message_id)?;
    let command_message_id = id16(&metadata.command_message_id)?;
    if exact_contract(
        envelope.contract.as_ref(),
        &attachment_preview_custody_delegated_contract_reference_v1(),
    ) {
        return Ok(DecodedCustodyResultV1::Delegated {
            message_id,
            envelope_sha256,
            command_message_id,
            payload: AttachmentPreviewCustodyDelegatedV1::decode(envelope.payload.as_slice())
                .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?,
        });
    }
    if exact_contract(
        envelope.contract.as_ref(),
        &attachment_preview_custody_delegation_rejected_contract_reference_v1(),
    ) {
        return Ok(DecodedCustodyResultV1::Rejected {
            message_id,
            envelope_sha256,
            command_message_id,
            payload: AttachmentPreviewCustodyDelegationRejectedV1::decode(
                envelope.payload.as_slice(),
            )
            .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?,
        });
    }
    Err(DeliveryDecodeErrorV1::WrongContract)
}

fn accepted(exact_bytes: &[u8]) -> Result<(DurableEnvelopeV1, [u8; 32]), DeliveryDecodeErrorV1> {
    let record = OutboxRecordV1::accept(exact_bytes.to_vec())
        .map_err(|_| DeliveryDecodeErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| DeliveryDecodeErrorV1::InvalidEnvelope)?;
    Ok((envelope, Sha256::digest(record.exact_bytes()).into()))
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

fn safety_state(value: i32) -> Result<AttachmentPreviewSafetyStateV1, DeliveryDecodeErrorV1> {
    match AttachmentSafetyStateV1::try_from(value)
        .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?
    {
        AttachmentSafetyStateV1::DescriptorOnly => {
            Ok(AttachmentPreviewSafetyStateV1::DescriptorOnly)
        }
        AttachmentSafetyStateV1::BlobPending => Ok(AttachmentPreviewSafetyStateV1::BlobPending),
        AttachmentSafetyStateV1::BlobAdmitted => Ok(AttachmentPreviewSafetyStateV1::BlobAdmitted),
        AttachmentSafetyStateV1::Quarantined => Ok(AttachmentPreviewSafetyStateV1::Quarantined),
        AttachmentSafetyStateV1::SafeForDelivery => {
            Ok(AttachmentPreviewSafetyStateV1::SafeForDelivery)
        }
        AttachmentSafetyStateV1::Rejected => Ok(AttachmentPreviewSafetyStateV1::Rejected),
        AttachmentSafetyStateV1::Unspecified => Err(DeliveryDecodeErrorV1::InvalidPayload),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], DeliveryDecodeErrorV1> {
    nonzero(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], DeliveryDecodeErrorV1> {
    nonzero(value)
}

fn nonzero<const N: usize>(value: &[u8]) -> Result<[u8; N], DeliveryDecodeErrorV1> {
    let value: [u8; N] = value
        .try_into()
        .map_err(|_| DeliveryDecodeErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(DeliveryDecodeErrorV1::InvalidPayload)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeliveryDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    InvalidPayload,
}
