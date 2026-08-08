//! Exact durable event decoding into Archive Inspection owner-local facts.

use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionCanonicalSafetyFactV1, ArchiveInspectionSafetyStateV1,
    ArchiveInspectionScanCandidateV1,
};
use makosh_attachment_archive_inspection_ingress::{
    archive_inspection_custody_delegated_contract_reference_v1,
    archive_inspection_custody_delegation_rejected_contract_reference_v1,
    wire::{ArchiveInspectionCustodyDelegatedV1, ArchiveInspectionCustodyDelegationRejectedV1},
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

pub struct DecodedArchiveScanCandidateV1 {
    pub fact: ArchiveInspectionScanCandidateV1,
    pub envelope_sha256: [u8; 32],
}

pub struct DecodedArchiveSafetyFactV1 {
    pub fact: ArchiveInspectionCanonicalSafetyFactV1,
    pub envelope_sha256: [u8; 32],
}

pub enum DecodedArchiveCustodyResultV1 {
    Delegated {
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: ArchiveInspectionCustodyDelegatedV1,
    },
    Rejected {
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: ArchiveInspectionCustodyDelegationRejectedV1,
    },
}

pub fn decode_archive_candidate_v1(
    exact_envelope_bytes: &[u8],
) -> Result<DecodedArchiveScanCandidateV1, ArchiveInspectionDeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_envelope_bytes)?;
    let expected = attachment_security_scan_candidate_observed_contract_reference_v1();
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(ArchiveInspectionDeliveryDecodeErrorV1::WrongContract);
    };
    let payload = AttachmentSecurityScanCandidateObservedV1::decode(envelope.payload.as_slice())
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?;
    if !exact_contract(envelope.contract.as_ref(), &expected)
        || metadata.observation_id != envelope.message_id
        || envelope.partition_key != payload.attachment_anchor_id
        || payload.declared_size == 0
        || !(1..=2_048).contains(&payload.custody_transfer_source_proof.len())
    {
        return Err(ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload);
    }
    Ok(DecodedArchiveScanCandidateV1 {
        fact: ArchiveInspectionScanCandidateV1 {
            message_id: id16(&envelope.message_id)?,
            attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
            blob_reference_id: id16(&payload.blob_reference_id)?,
            declared_size: payload.declared_size,
            blob_receipt_sha256: id32(&payload.blob_receipt_sha256)?,
            custody_transfer_source_proof: payload.custody_transfer_source_proof,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        envelope_sha256,
    })
}

pub fn decode_archive_safety_v1(
    exact_envelope_bytes: &[u8],
) -> Result<Option<DecodedArchiveSafetyFactV1>, ArchiveInspectionDeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_envelope_bytes)?;
    let expected = communication_attachment_safety_state_changed_contract_reference_v1();
    let Some(Semantics::Event(_)) = envelope.semantics.as_ref() else {
        return Err(ArchiveInspectionDeliveryDecodeErrorV1::WrongContract);
    };
    let payload = AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice())
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?;
    if !exact_contract(envelope.contract.as_ref(), &expected)
        || envelope.partition_key != payload.attachment_anchor_id
    {
        return Err(ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload);
    }
    let expected_state = safety_state(payload.expected_state)?;
    let next_state = safety_state(payload.next_state)?;
    if !matches!(
        next_state,
        ArchiveInspectionSafetyStateV1::SafeForDelivery
            | ArchiveInspectionSafetyStateV1::Quarantined
            | ArchiveInspectionSafetyStateV1::Rejected
    ) {
        return Ok(None);
    }
    Ok(Some(DecodedArchiveSafetyFactV1 {
        fact: ArchiveInspectionCanonicalSafetyFactV1 {
            message_id: id16(&envelope.message_id)?,
            attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
            expected_state,
            next_state,
            evidence_id: id16(&payload.evidence_id)?,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        envelope_sha256,
    }))
}

pub fn decode_archive_custody_result_v1(
    exact_envelope_bytes: &[u8],
) -> Result<DecodedArchiveCustodyResultV1, ArchiveInspectionDeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted(exact_envelope_bytes)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(ArchiveInspectionDeliveryDecodeErrorV1::WrongContract);
    };
    let message_id = id16(&envelope.message_id)?;
    let command_message_id = id16(&metadata.command_message_id)?;
    if exact_contract(
        envelope.contract.as_ref(),
        &archive_inspection_custody_delegated_contract_reference_v1(),
    ) {
        let payload = ArchiveInspectionCustodyDelegatedV1::decode(envelope.payload.as_slice())
            .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?;
        return Ok(DecodedArchiveCustodyResultV1::Delegated {
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
        });
    }
    if exact_contract(
        envelope.contract.as_ref(),
        &archive_inspection_custody_delegation_rejected_contract_reference_v1(),
    ) {
        let payload =
            ArchiveInspectionCustodyDelegationRejectedV1::decode(envelope.payload.as_slice())
                .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?;
        return Ok(DecodedArchiveCustodyResultV1::Rejected {
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
        });
    }
    Err(ArchiveInspectionDeliveryDecodeErrorV1::WrongContract)
}

fn accepted(
    exact_envelope_bytes: &[u8],
) -> Result<(DurableEnvelopeV1, [u8; 32]), ArchiveInspectionDeliveryDecodeErrorV1> {
    let record = OutboxRecordV1::accept(exact_envelope_bytes.to_vec())
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidEnvelope)?;
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

fn safety_state(
    value: i32,
) -> Result<ArchiveInspectionSafetyStateV1, ArchiveInspectionDeliveryDecodeErrorV1> {
    match AttachmentSafetyStateV1::try_from(value)
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?
    {
        AttachmentSafetyStateV1::DescriptorOnly => {
            Ok(ArchiveInspectionSafetyStateV1::DescriptorOnly)
        }
        AttachmentSafetyStateV1::BlobPending => Ok(ArchiveInspectionSafetyStateV1::BlobPending),
        AttachmentSafetyStateV1::BlobAdmitted => Ok(ArchiveInspectionSafetyStateV1::BlobAdmitted),
        AttachmentSafetyStateV1::Quarantined => Ok(ArchiveInspectionSafetyStateV1::Quarantined),
        AttachmentSafetyStateV1::SafeForDelivery => {
            Ok(ArchiveInspectionSafetyStateV1::SafeForDelivery)
        }
        AttachmentSafetyStateV1::Rejected => Ok(ArchiveInspectionSafetyStateV1::Rejected),
        AttachmentSafetyStateV1::Unspecified => {
            Err(ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)
        }
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ArchiveInspectionDeliveryDecodeErrorV1> {
    nonzero(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ArchiveInspectionDeliveryDecodeErrorV1> {
    nonzero(value)
}

fn nonzero<const N: usize>(
    value: &[u8],
) -> Result<[u8; N], ArchiveInspectionDeliveryDecodeErrorV1> {
    let value: [u8; N] = value
        .try_into()
        .map_err(|_| ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(ArchiveInspectionDeliveryDecodeErrorV1::InvalidPayload)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionDeliveryDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    InvalidPayload,
}
