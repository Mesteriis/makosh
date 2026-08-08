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
    ATTACHMENT_PREVIEW_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
    ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
    ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
    ATTACHMENT_PREVIEW_INGRESS_CONTRACT_MAJOR_V1, ATTACHMENT_PREVIEW_INGRESS_CONTRACT_REVISION_V1,
    ATTACHMENT_PREVIEW_INGRESS_OWNER_V1, ATTACHMENT_PREVIEW_INGRESS_SCHEMA_SHA256,
    ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1, ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1,
    ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1,
    attachment_preview_custody_delegated_message_id_v1,
    attachment_preview_custody_delegation_rejected_message_id_v1,
    wire::{
        AttachmentPreviewCustodyDelegatedV1, AttachmentPreviewCustodyDelegationRejectedV1,
        RequestAttachmentPreviewCustodyDelegationV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentPreviewCustodyEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewCustodyEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_request_attachment_preview_custody_delegation_outbox_record_v1(
    payload: RequestAttachmentPreviewCustodyDelegationV1,
    deadline_unix_seconds: i64,
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_request(&payload)?;
    let run_id = id16(&payload.preview_run_id)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        request_id,
        run_id,
        &[],
        ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: request_id.to_vec(),
            target_capability: ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"attachment-preview-custody-delegation-v1".as_slice(),
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

pub fn build_attachment_preview_custody_delegated_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentPreviewCustodyDelegatedV1,
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_delegated(&payload)?;
    build_result(
        command_message_id,
        request_id,
        id16(&payload.preview_run_id)?,
        ATTACHMENT_PREVIEW_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        attachment_preview_custody_delegated_message_id_v1(request_id),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_attachment_preview_custody_delegation_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentPreviewCustodyDelegationRejectedV1,
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = id16(&payload.request_id)?;
    id16(&payload.attachment_anchor_id)?;
    if payload.code == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        command_message_id,
        request_id,
        id16(&payload.preview_run_id)?,
        ATTACHMENT_PREVIEW_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        attachment_preview_custody_delegation_rejected_message_id_v1(request_id),
        payload.encode_to_vec(),
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    command_message_id: [u8; 16],
    request_id: [u8; 16],
    run_id: [u8; 16],
    contract_name: &str,
    outcome: ResultOutcomeV1,
    message_id: [u8; 16],
    payload: Vec<u8>,
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    valid_id(command_message_id)?;
    build_envelope(
        message_id,
        run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: request_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build_envelope(
    message_id: [u8; 16],
    partition_key: [u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: ATTACHMENT_PREVIEW_INGRESS_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: ATTACHMENT_PREVIEW_INGRESS_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_PREVIEW_INGRESS_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_PREVIEW_INGRESS_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: Sha256::digest(context.runtime_instance_id.as_bytes())[..16]
                .to_vec(),
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
        .map_err(|_| AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_request(
    payload: &RequestAttachmentPreviewCustodyDelegationV1,
) -> Result<[u8; 16], AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    id16(&payload.preview_run_id)?;
    id16(&payload.attachment_anchor_id)?;
    id16(&payload.candidate_message_id)?;
    sha256(&payload.candidate_envelope_sha256)?;
    id16(&payload.safety_message_id)?;
    id16(&payload.safety_evidence_id)?;
    if !valid_owner(&payload.logical_owner_id) {
        return Err(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_delegated(
    payload: &AttachmentPreviewCustodyDelegatedV1,
) -> Result<[u8; 16], AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    id16(&payload.preview_run_id)?;
    id16(&payload.attachment_anchor_id)?;
    id16(&payload.candidate_message_id)?;
    id16(&payload.safety_message_id)?;
    id16(&payload.source_reference_id)?;
    sha256(&payload.receipt_sha256)?;
    if !(1..=ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1).contains(&payload.declared_size)
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len() > ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_context(
    context: &AttachmentPreviewCustodyEnvelopeContextV1,
) -> Result<(), AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
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
        Err(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload)?;
    valid_id(value)?;
    Ok(value)
}

fn sha256(bytes: &[u8]) -> Result<[u8; 32], AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    let value: [u8; 32] = bytes
        .try_into()
        .map_err(|_| AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_id(value: [u8; 16]) -> Result<(), AttachmentPreviewCustodyEnvelopeBuildErrorV1> {
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(())
        .ok_or(AttachmentPreviewCustodyEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn timestamp(context: &AttachmentPreviewCustodyEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn outbox_error(_: OutboxRecordError) -> AttachmentPreviewCustodyEnvelopeBuildErrorV1 {
    AttachmentPreviewCustodyEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    #[test]
    fn command_is_exact_bounded_and_private() {
        let payload = RequestAttachmentPreviewCustodyDelegationV1 {
            request_id: vec![1; 16],
            preview_run_id: vec![2; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            candidate_envelope_sha256: vec![5; 32],
            safety_message_id: vec![6; 16],
            safety_evidence_id: vec![7; 16],
            logical_owner_id: "owner-1".to_owned(),
        };
        let record = build_request_attachment_preview_custody_delegation_outbox_record_v1(
            payload,
            1_800_000_030,
            &AttachmentPreviewCustodyEnvelopeContextV1 {
                module_id: "makosh-attachment-preview-runtime".to_owned(),
                runtime_instance_id: "preview-1".to_owned(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).unwrap();
        let Semantics::Command(command) = envelope.semantics.unwrap() else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1
        );
    }
}
