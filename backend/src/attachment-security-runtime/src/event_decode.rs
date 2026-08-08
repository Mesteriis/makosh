//! Exact durable-envelope decoding for the engine's two admitted inputs.

use makosh_attachment_security_contract::{
    admission::attachment_security_scan_candidate_observed_contract_reference_v1,
    v1::AttachmentSecurityScanCandidateObservedV1,
};
use makosh_attachment_security_core::{
    AttachmentSecurityCanonicalStateFactV1, AttachmentSecurityScanCandidateV1,
    CanonicalAttachmentSafetyStateV1,
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

pub struct DecodedScanCandidateV1 {
    pub fact: AttachmentSecurityScanCandidateV1,
    pub envelope_sha256: [u8; 32],
}

pub struct DecodedCanonicalStateV1 {
    pub fact: AttachmentSecurityCanonicalStateFactV1,
    pub envelope_sha256: [u8; 32],
}

pub fn decode_scan_candidate_v1(
    exact_envelope_bytes: &[u8],
) -> Result<DecodedScanCandidateV1, AttachmentSecurityDeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted_envelope(exact_envelope_bytes)?;
    let expected = attachment_security_scan_candidate_observed_contract_reference_v1();
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::WrongContract);
    };
    if !exact_contract(envelope.contract.as_ref(), &expected)
        || metadata.observation_id != envelope.message_id
        || metadata.source_cursor_sha256.len() != 32
    {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::WrongContract);
    }
    let payload = AttachmentSecurityScanCandidateObservedV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)?;
    let message_id = id16(&envelope.message_id)?;
    let attachment_anchor_id = id16(&payload.attachment_anchor_id)?;
    let blob_reference_id = id16(&payload.blob_reference_id)?;
    let blob_receipt_sha256 = id32(&payload.blob_receipt_sha256)?;
    let causation_message_id = id16(&envelope.causation_message_id)?;
    let correlation_id = id16(&envelope.correlation_id)?;
    if envelope.partition_key != payload.attachment_anchor_id
        || payload.declared_size == 0
        || !(1..=2_048).contains(&payload.custody_transfer_source_proof.len())
        || !occurred_at_matches(
            metadata.occurred_at.as_ref(),
            payload.observed_at_unix_seconds,
        )
    {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload);
    }
    Ok(DecodedScanCandidateV1 {
        fact: AttachmentSecurityScanCandidateV1 {
            message_id,
            attachment_anchor_id,
            blob_reference_id,
            declared_size: payload.declared_size,
            blob_receipt_sha256,
            custody_transfer_source_proof: payload.custody_transfer_source_proof,
            causation_message_id,
            correlation_id,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        envelope_sha256,
    })
}

pub fn decode_canonical_state_v1(
    exact_envelope_bytes: &[u8],
) -> Result<Option<DecodedCanonicalStateV1>, AttachmentSecurityDeliveryDecodeErrorV1> {
    let (envelope, envelope_sha256) = accepted_envelope(exact_envelope_bytes)?;
    let expected = communication_attachment_safety_state_changed_contract_reference_v1();
    let Some(Semantics::Event(metadata)) = envelope.semantics.as_ref() else {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::WrongContract);
    };
    if !exact_contract(envelope.contract.as_ref(), &expected) {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::WrongContract);
    }
    let payload = AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)?;
    let message_id = id16(&envelope.message_id)?;
    let attachment_anchor_id = id16(&payload.attachment_anchor_id)?;
    let evidence_id = id16(&payload.evidence_id)?;
    let correlation_id = id16(&envelope.correlation_id)?;
    let expected_state = canonical_state(payload.expected_state)?;
    let next_state = canonical_state(payload.next_state)?;
    if envelope.partition_key != payload.attachment_anchor_id
        || !valid_owner_transition(expected_state, next_state)
        || !occurred_at_matches(
            metadata.occurred_at.as_ref(),
            payload.observed_at_unix_seconds,
        )
    {
        return Err(AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload);
    }
    if expected_state != CanonicalAttachmentSafetyStateV1::BlobPending
        || next_state != CanonicalAttachmentSafetyStateV1::BlobAdmitted
    {
        return Ok(None);
    }
    Ok(Some(DecodedCanonicalStateV1 {
        fact: AttachmentSecurityCanonicalStateFactV1 {
            message_id,
            attachment_anchor_id,
            expected_state,
            next_state,
            evidence_id,
            correlation_id,
            observed_at_unix_seconds: payload.observed_at_unix_seconds,
        },
        envelope_sha256,
    }))
}

const fn valid_owner_transition(
    expected: CanonicalAttachmentSafetyStateV1,
    next: CanonicalAttachmentSafetyStateV1,
) -> bool {
    matches!(
        (expected, next),
        (
            CanonicalAttachmentSafetyStateV1::DescriptorOnly,
            CanonicalAttachmentSafetyStateV1::BlobPending
        ) | (
            CanonicalAttachmentSafetyStateV1::BlobPending,
            CanonicalAttachmentSafetyStateV1::BlobAdmitted
        ) | (
            CanonicalAttachmentSafetyStateV1::BlobAdmitted,
            CanonicalAttachmentSafetyStateV1::SafeForDelivery
        ) | (
            CanonicalAttachmentSafetyStateV1::DescriptorOnly
                | CanonicalAttachmentSafetyStateV1::BlobPending
                | CanonicalAttachmentSafetyStateV1::BlobAdmitted,
            CanonicalAttachmentSafetyStateV1::Quarantined
                | CanonicalAttachmentSafetyStateV1::Rejected
        )
    )
}

fn accepted_envelope(
    exact_envelope_bytes: &[u8],
) -> Result<(DurableEnvelopeV1, [u8; 32]), AttachmentSecurityDeliveryDecodeErrorV1> {
    let record = OutboxRecordV1::accept(exact_envelope_bytes.to_vec())
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidEnvelope)?;
    Ok((envelope, Sha256::digest(record.exact_bytes()).into()))
}

fn canonical_state(
    value: i32,
) -> Result<CanonicalAttachmentSafetyStateV1, AttachmentSecurityDeliveryDecodeErrorV1> {
    match AttachmentSafetyStateV1::try_from(value)
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)?
    {
        AttachmentSafetyStateV1::DescriptorOnly => {
            Ok(CanonicalAttachmentSafetyStateV1::DescriptorOnly)
        }
        AttachmentSafetyStateV1::BlobPending => Ok(CanonicalAttachmentSafetyStateV1::BlobPending),
        AttachmentSafetyStateV1::BlobAdmitted => Ok(CanonicalAttachmentSafetyStateV1::BlobAdmitted),
        AttachmentSafetyStateV1::Quarantined => Ok(CanonicalAttachmentSafetyStateV1::Quarantined),
        AttachmentSafetyStateV1::SafeForDelivery => {
            Ok(CanonicalAttachmentSafetyStateV1::SafeForDelivery)
        }
        AttachmentSafetyStateV1::Rejected => Ok(CanonicalAttachmentSafetyStateV1::Rejected),
        AttachmentSafetyStateV1::Unspecified => {
            Err(AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)
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

fn occurred_at_matches(value: Option<&prost_types::Timestamp>, seconds: i64) -> bool {
    value.is_some_and(|value| value.seconds == seconds && value.nanos == 0)
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentSecurityDeliveryDecodeErrorV1> {
    nonzero_array(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], AttachmentSecurityDeliveryDecodeErrorV1> {
    nonzero_array(value)
}

fn nonzero_array<const N: usize>(
    value: &[u8],
) -> Result<[u8; N], AttachmentSecurityDeliveryDecodeErrorV1> {
    let value: [u8; N] = value
        .try_into()
        .map_err(|_| AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)?;
    (!value.iter().all(|byte| *byte == 0))
        .then_some(value)
        .ok_or(AttachmentSecurityDeliveryDecodeErrorV1::InvalidPayload)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityDeliveryDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    InvalidPayload,
}

#[cfg(test)]
mod tests {
    use makosh_attachment_security_contract::{
        AttachmentSecurityObservationContextV1, AttachmentSecurityScanCandidateFactV1,
        build_attachment_security_scan_candidate_outbox_record_v1,
    };
    use makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateChangedV1;
    use makosh_events_protocol::v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1,
    };
    use prost_types::Timestamp;

    use super::*;

    #[test]
    fn candidate_decoder_preserves_exact_envelope_identity_and_blob_receipt() {
        let record = build_attachment_security_scan_candidate_outbox_record_v1(
            &AttachmentSecurityScanCandidateFactV1 {
                attachment_anchor_id: [1; 16],
                blob_reference_id: [2; 16],
                declared_size: 42,
                blob_receipt_sha256: [3; 32],
                custody_transfer_source_proof: vec![9; 64],
                source_observation_id: [4; 16],
                correlation_id: [5; 16],
                observed_at_unix_seconds: 1_700_000_000,
            },
            &AttachmentSecurityObservationContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 1,
                module_id: "makosh-mail-runtime".to_owned(),
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("candidate");

        let decoded = decode_scan_candidate_v1(record.exact_bytes()).expect("decoded");
        assert_eq!(decoded.fact.attachment_anchor_id, [1; 16]);
        assert_eq!(decoded.fact.blob_reference_id, [2; 16]);
        assert_eq!(decoded.fact.blob_receipt_sha256, [3; 32]);
        assert_eq!(decoded.fact.custody_transfer_source_proof, [9; 64]);
        assert_eq!(decoded.fact.causation_message_id, [4; 16]);
        assert_eq!(
            decoded.envelope_sha256,
            <[u8; 32]>::from(Sha256::digest(record.exact_bytes()))
        );
    }

    #[test]
    fn canonical_decoder_accepts_only_the_blob_pending_to_admitted_owner_event() {
        let contract = communication_attachment_safety_state_changed_contract_reference_v1();
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: vec![6; 16],
            contract: Some(ContractRefV1 {
                owner: contract.owner,
                name: contract.name,
                major: contract.major,
                revision: contract.revision,
                schema_sha256: contract.schema_sha256,
            }),
            source: Some(SourceRefV1 {
                module_id: "makosh-communications-runtime".to_owned(),
                runtime_instance_id: vec![7; 16],
                runtime_generation: 3,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1_700_000_002,
                nanos: 0,
            }),
            partition_key: vec![1; 16],
            causation_message_id: vec![8; 16],
            correlation_id: vec![5; 16],
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::Module as i32,
                actor_id: b"communications-runtime".to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: b"communications-runtime".to_vec(),
                epoch: 3,
            }),
            semantics: Some(Semantics::Event(EventMetadataV1 {
                occurred_at: Some(Timestamp {
                    seconds: 1_700_000_000,
                    nanos: 0,
                }),
            })),
            payload: AttachmentSafetyStateChangedV1 {
                attachment_anchor_id: vec![1; 16],
                expected_state: AttachmentSafetyStateV1::BlobPending as i32,
                next_state: AttachmentSafetyStateV1::BlobAdmitted as i32,
                evidence_id: vec![9; 16],
                observed_at_unix_seconds: 1_700_000_000,
            }
            .encode_to_vec(),
        };
        let exact = envelope.encode_to_vec();

        let decoded = decode_canonical_state_v1(&exact)
            .expect("canonical state")
            .expect("scan admission transition");
        assert_eq!(decoded.fact.message_id, [6; 16]);
        assert_eq!(decoded.fact.attachment_anchor_id, [1; 16]);
        assert_eq!(
            decoded.fact.expected_state,
            CanonicalAttachmentSafetyStateV1::BlobPending
        );
        assert_eq!(
            decoded.fact.next_state,
            CanonicalAttachmentSafetyStateV1::BlobAdmitted
        );
        assert_eq!(decoded.fact.evidence_id, [9; 16]);
        assert_eq!(decoded.fact.correlation_id, [5; 16]);

        let mut irrelevant = envelope;
        irrelevant.message_id = vec![10; 16];
        irrelevant.payload = AttachmentSafetyStateChangedV1 {
            attachment_anchor_id: vec![1; 16],
            expected_state: AttachmentSafetyStateV1::DescriptorOnly as i32,
            next_state: AttachmentSafetyStateV1::BlobPending as i32,
            evidence_id: vec![11; 16],
            observed_at_unix_seconds: 1_700_000_000,
        }
        .encode_to_vec();
        assert!(
            decode_canonical_state_v1(&irrelevant.encode_to_vec())
                .expect("valid owner transition")
                .is_none(),
            "valid non-scan lifecycle events must be acknowledged without joining a job"
        );
    }
}
