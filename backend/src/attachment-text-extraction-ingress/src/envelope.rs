use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1, ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256,
    ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1, ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1,
    attachment_text_custody_delegated_message_id_v1,
    attachment_text_custody_delegation_rejected_message_id_v1,
    wire::{
        AttachmentTextCustodyDelegatedV1, AttachmentTextCustodyDelegationRejectedV1,
        RequestAttachmentTextCustodyDelegationV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextCustodyEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextCustodyEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

struct ResultEnvelopeV1 {
    command_message_id: [u8; 16],
    request_id: [u8; 16],
    extraction_run_id: [u8; 16],
    label: &'static [u8],
    contract_name: &'static str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
}

pub fn build_request_attachment_text_custody_delegation_outbox_record_v1(
    payload: RequestAttachmentTextCustodyDelegationV1,
    deadline_unix_seconds: i64,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTextCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_request(&payload)?;
    let extraction_run_id = id16(&payload.extraction_run_id)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        request_id,
        &extraction_run_id,
        &[],
        ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: request_id.to_vec(),
            target_capability: ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1
                .to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"attachment-text-custody-delegation-v1".as_slice(),
                    &request_id,
                ]
                .concat(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_attachment_text_custody_delegated_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentTextCustodyDelegatedV1,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTextCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_delegated(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        ResultEnvelopeV1 {
            command_message_id,
            request_id,
            extraction_run_id: id16(&payload.extraction_run_id)?,
            label: b"delegated",
            contract_name: ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Succeeded,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_attachment_text_custody_delegation_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentTextCustodyDelegationRejectedV1,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTextCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = id16(&payload.request_id)?;
    let extraction_run_id = id16(&payload.extraction_run_id)?;
    id16(&payload.attachment_anchor_id)?;
    if !valid_id(&command_message_id)
        || payload.code == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
    {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        ResultEnvelopeV1 {
            command_message_id,
            request_id,
            extraction_run_id,
            label: b"rejected",
            contract_name: ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Rejected,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

fn build_result(
    result: ResultEnvelopeV1,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTextCustodyEnvelopeBuildErrorV1> {
    build_envelope(
        if result.label == b"delegated" {
            attachment_text_custody_delegated_message_id_v1(result.request_id)
        } else {
            attachment_text_custody_delegation_rejected_message_id_v1(result.request_id)
        },
        &result.extraction_run_id,
        &result.command_message_id,
        result.contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: result.request_id.to_vec(),
            command_message_id: result.command_message_id.to_vec(),
            outcome: result.outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        result.payload,
        context,
    )
}

fn build_envelope(
    message_id: [u8; 16],
    partition_key: &[u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTextCustodyEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: partition_key.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: partition_key.to_vec(),
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
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_request(
    payload: &RequestAttachmentTextCustodyDelegationV1,
) -> Result<[u8; 16], AttachmentTextCustodyEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    id16(&payload.extraction_run_id)?;
    id16(&payload.attachment_anchor_id)?;
    id16(&payload.candidate_message_id)?;
    sha256(&payload.candidate_envelope_sha256)?;
    id16(&payload.safety_message_id)?;
    id16(&payload.safety_evidence_id)?;
    if !valid_logical_owner_id(&payload.logical_owner_id) {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_delegated(
    payload: &AttachmentTextCustodyDelegatedV1,
) -> Result<[u8; 16], AttachmentTextCustodyEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    id16(&payload.extraction_run_id)?;
    id16(&payload.attachment_anchor_id)?;
    id16(&payload.candidate_message_id)?;
    id16(&payload.safety_message_id)?;
    id16(&payload.source_reference_id)?;
    sha256(&payload.receipt_sha256)?;
    if !(1..=ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1).contains(&payload.declared_size)
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len()
            > ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1
        || !valid_logical_owner_id(&payload.logical_owner_id)
    {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_context(
    context: &AttachmentTextCustodyEnvelopeContextV1,
) -> Result<(), AttachmentTextCustodyEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || !context.module_id.is_ascii()
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], AttachmentTextCustodyEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload)
}

fn sha256(bytes: &[u8]) -> Result<[u8; 32], AttachmentTextCustodyEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn timestamp(context: &AttachmentTextCustodyEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> AttachmentTextCustodyEnvelopeBuildErrorV1 {
    AttachmentTextCustodyEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context(module_id: &str) -> AttachmentTextCustodyEnvelopeContextV1 {
        AttachmentTextCustodyEnvelopeContextV1 {
            module_id: module_id.to_owned(),
            runtime_instance_id: format!("{module_id}-1"),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    fn request() -> RequestAttachmentTextCustodyDelegationV1 {
        RequestAttachmentTextCustodyDelegationV1 {
            request_id: vec![1; 16],
            extraction_run_id: vec![2; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            candidate_envelope_sha256: vec![5; 32],
            safety_message_id: vec![6; 16],
            safety_evidence_id: vec![7; 16],
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn command_is_bounded_and_targets_exact_source_capability() {
        let record = build_request_attachment_text_custody_delegation_outbox_record_v1(
            request(),
            1_800_000_030,
            &context("makosh-attachment-text-extraction-runtime"),
        )
        .expect("valid command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Semantics::Command(command) = envelope.semantics.expect("semantics") else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1
        );
        assert_eq!(envelope.partition_key, vec![2; 16]);
        assert!(envelope.causation_message_id.is_empty());
    }

    #[test]
    fn delegated_result_is_command_linked_and_proof_bounded() {
        let payload = AttachmentTextCustodyDelegatedV1 {
            request_id: vec![1; 16],
            extraction_run_id: vec![2; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            safety_message_id: vec![6; 16],
            source_reference_id: vec![8; 16],
            declared_size: 1024,
            receipt_sha256: vec![9; 32],
            custody_transfer_source_proof: vec![10; 64],
            logical_owner_id: "owner-1".to_owned(),
        };
        let record = build_attachment_text_custody_delegated_outbox_record_v1(
            [1; 16],
            payload,
            &context("makosh-attachment-security-runtime"),
        )
        .expect("valid result");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Semantics::Result(result) = envelope.semantics.expect("semantics") else {
            panic!("result");
        };
        assert_eq!(result.command_id, vec![1; 16]);
        assert_eq!(result.command_message_id, vec![1; 16]);
        assert_eq!(envelope.causation_message_id, vec![1; 16]);
    }

    #[test]
    fn private_result_rejects_empty_or_oversized_proof() {
        let mut payload = AttachmentTextCustodyDelegatedV1 {
            request_id: vec![1; 16],
            extraction_run_id: vec![2; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            safety_message_id: vec![6; 16],
            source_reference_id: vec![8; 16],
            declared_size: 1024,
            receipt_sha256: vec![9; 32],
            custody_transfer_source_proof: Vec::new(),
            logical_owner_id: "owner-1".to_owned(),
        };
        assert_eq!(
            build_attachment_text_custody_delegated_outbox_record_v1(
                [1; 16],
                payload.clone(),
                &context("makosh-attachment-security-runtime"),
            ),
            Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload)
        );
        payload.custody_transfer_source_proof =
            vec![10; ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1 + 1];
        assert_eq!(
            build_attachment_text_custody_delegated_outbox_record_v1(
                [1; 16],
                payload,
                &context("makosh-attachment-security-runtime"),
            ),
            Err(AttachmentTextCustodyEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
