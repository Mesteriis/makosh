use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256,
    admission::{
        ATTACHMENT_SECURITY_CONTRACT_MAJOR, ATTACHMENT_SECURITY_CONTRACT_OWNER,
        ATTACHMENT_SECURITY_CONTRACT_REVISION, ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME,
    },
    v1::AttachmentSecurityScanCandidateObservedV1,
};

pub const ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1: u64 = 64 * 1024 * 1024;
pub const ATTACHMENT_SECURITY_MAX_CUSTODY_SOURCE_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityObservationContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub module_id: String,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityScanCandidateFactV1 {
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub source_observation_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityObservationBuildErrorV1 {
    InvalidContext,
    InvalidCandidate,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_attachment_security_scan_candidate_outbox_record_v1(
    fact: &AttachmentSecurityScanCandidateFactV1,
    context: &AttachmentSecurityObservationContextV1,
) -> Result<OutboxRecordV1, AttachmentSecurityObservationBuildErrorV1> {
    validate_context(context)?;
    if !valid_identifier(&fact.attachment_anchor_id)
        || !valid_identifier(&fact.blob_reference_id)
        || fact.declared_size == 0
        || fact.declared_size > ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1
        || !valid_sha256(&fact.blob_receipt_sha256)
        || !(1..=ATTACHMENT_SECURITY_MAX_CUSTODY_SOURCE_PROOF_BYTES_V1)
            .contains(&fact.custody_transfer_source_proof.len())
        || !valid_identifier(&fact.source_observation_id)
        || !valid_identifier(&fact.correlation_id)
        || !valid_timestamp(fact.observed_at_unix_seconds, 0)
    {
        return Err(AttachmentSecurityObservationBuildErrorV1::InvalidCandidate);
    }

    let message_id = candidate_message_id(fact);
    let source_cursor_sha256 = candidate_source_cursor_sha256(fact);
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let payload = AttachmentSecurityScanCandidateObservedV1 {
        attachment_anchor_id: fact.attachment_anchor_id.to_vec(),
        blob_reference_id: fact.blob_reference_id.to_vec(),
        declared_size: fact.declared_size,
        blob_receipt_sha256: fact.blob_receipt_sha256.to_vec(),
        observed_at_unix_seconds: fact.observed_at_unix_seconds,
        custody_transfer_source_proof: fact.custody_transfer_source_proof.clone(),
    }
    .encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: ATTACHMENT_SECURITY_CONTRACT_OWNER.to_owned(),
            name: ATTACHMENT_SECURITY_SCAN_CANDIDATE_CONTRACT_NAME.to_owned(),
            major: ATTACHMENT_SECURITY_CONTRACT_MAJOR,
            revision: ATTACHMENT_SECURITY_CONTRACT_REVISION,
            schema_sha256: ATTACHMENT_SECURITY_SCAN_CANDIDATE_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: fact.attachment_anchor_id.to_vec(),
        causation_message_id: fact.source_observation_id.to_vec(),
        correlation_id: fact.correlation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: message_id.to_vec(),
            observed_at: Some(timestamp),
            occurred_at: Some(Timestamp {
                seconds: fact.observed_at_unix_seconds,
                nanos: 0,
            }),
            source_cursor_sha256: source_cursor_sha256.to_vec(),
            source_sequence: None,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| AttachmentSecurityObservationBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &AttachmentSecurityObservationContextV1,
) -> Result<(), AttachmentSecurityObservationBuildErrorV1> {
    if context.runtime_generation == 0
        || !valid_runtime_identifier(&context.module_id)
        || !valid_runtime_identifier(&context.runtime_instance_id)
        || !valid_timestamp(context.recorded_at_unix_seconds, context.recorded_at_nanos)
    {
        return Err(AttachmentSecurityObservationBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn candidate_message_id(fact: &AttachmentSecurityScanCandidateFactV1) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-security.scan-candidate-message.v2\0");
    hasher.update(fact.attachment_anchor_id);
    hasher.update(fact.blob_reference_id);
    hasher.update(fact.declared_size.to_be_bytes());
    hasher.update(fact.blob_receipt_sha256);
    hasher.update(Sha256::digest(&fact.custody_transfer_source_proof));
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn candidate_source_cursor_sha256(fact: &AttachmentSecurityScanCandidateFactV1) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-security.scan-candidate-source.v2\0");
    hasher.update(fact.attachment_anchor_id);
    hasher.update(fact.blob_reference_id);
    hasher.update(fact.declared_size.to_be_bytes());
    hasher.update(fact.blob_receipt_sha256);
    hasher.update(Sha256::digest(&fact.custody_transfer_source_proof));
    hasher.finalize().into()
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn valid_identifier(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_runtime_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_timestamp(seconds: i64, nanos: i32) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&seconds) && (0..1_000_000_000).contains(&nanos)
}

fn outbox_error(_: OutboxRecordError) -> AttachmentSecurityObservationBuildErrorV1 {
    AttachmentSecurityObservationBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::validation::envelope::decode_envelope_v1;

    #[test]
    fn candidate_is_anchor_partitioned_and_contains_only_bounded_blob_identity() {
        let record =
            build_attachment_security_scan_candidate_outbox_record_v1(&candidate(), &context())
                .expect("candidate");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let payload =
            AttachmentSecurityScanCandidateObservedV1::decode(envelope.payload.as_slice())
                .expect("payload");

        assert_eq!(envelope.partition_key, [1; 16]);
        assert_eq!(envelope.causation_message_id, [4; 16]);
        assert_eq!(envelope.correlation_id, [5; 16]);
        assert_eq!(payload.attachment_anchor_id, [1; 16]);
        assert_eq!(payload.blob_reference_id, [2; 16]);
        assert_eq!(payload.blob_receipt_sha256, [3; 32]);
        assert_eq!(payload.declared_size, 42);
        assert_eq!(payload.custody_transfer_source_proof, [6; 64]);
    }

    #[test]
    fn candidate_rejects_missing_receipt_and_payload_expanded_size() {
        let mut missing_receipt = candidate();
        missing_receipt.blob_receipt_sha256 = [0; 32];
        assert_eq!(
            build_attachment_security_scan_candidate_outbox_record_v1(&missing_receipt, &context())
                .expect_err("missing receipt"),
            AttachmentSecurityObservationBuildErrorV1::InvalidCandidate
        );

        let mut oversized = candidate();
        oversized.declared_size = ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1 + 1;
        assert_eq!(
            build_attachment_security_scan_candidate_outbox_record_v1(&oversized, &context())
                .expect_err("oversized candidate"),
            AttachmentSecurityObservationBuildErrorV1::InvalidCandidate
        );

        let mut missing_proof = candidate();
        missing_proof.custody_transfer_source_proof.clear();
        assert_eq!(
            build_attachment_security_scan_candidate_outbox_record_v1(&missing_proof, &context())
                .expect_err("missing custody proof"),
            AttachmentSecurityObservationBuildErrorV1::InvalidCandidate
        );
    }

    fn candidate() -> AttachmentSecurityScanCandidateFactV1 {
        AttachmentSecurityScanCandidateFactV1 {
            attachment_anchor_id: [1; 16],
            blob_reference_id: [2; 16],
            declared_size: 42,
            blob_receipt_sha256: [3; 32],
            custody_transfer_source_proof: vec![6; 64],
            source_observation_id: [4; 16],
            correlation_id: [5; 16],
            observed_at_unix_seconds: 1_700_000_000,
        }
    }

    fn context() -> AttachmentSecurityObservationContextV1 {
        AttachmentSecurityObservationContextV1 {
            runtime_instance_id: "mail-runtime-test".to_owned(),
            runtime_generation: 2,
            module_id: "mail-runtime".to_owned(),
            recorded_at_unix_seconds: 1_700_000_001,
            recorded_at_nanos: 0,
        }
    }
}
