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
    ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1,
    ATTACHMENT_TRANSLATION_INGRESS_CONTRACT_MAJOR_V1,
    ATTACHMENT_TRANSLATION_INGRESS_CONTRACT_REVISION_V1, ATTACHMENT_TRANSLATION_INGRESS_OWNER_V1,
    ATTACHMENT_TRANSLATION_INGRESS_SCHEMA_SHA256, ATTACHMENT_TRANSLATION_MAX_PROOF_BYTES_V1,
    ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1,
    ATTACHMENT_TRANSLATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
    ATTACHMENT_TRANSLATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
    ATTACHMENT_TRANSLATION_SOURCE_REQUESTED_CONTRACT_NAME_V1,
    attachment_translation_source_prepared_message_id_v1,
    attachment_translation_source_rejected_message_id_v1,
    attachment_translation_source_request_id_v1,
    wire::{
        AttachmentTranslationSourcePreparedV1, AttachmentTranslationSourceRejectedV1,
        RequestAttachmentTranslationSourceV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationSourceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationSourceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_request_attachment_translation_source_outbox_record_v1(
    payload: RequestAttachmentTranslationSourceV1,
    deadline_unix_seconds: i64,
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_request(&payload)?;
    let run_id = id16(&payload.translation_run_id)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        request_id,
        run_id,
        &[],
        ATTACHMENT_TRANSLATION_SOURCE_REQUESTED_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: request_id.to_vec(),
            target_capability: ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1
                .to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"attachment-translation-source-request-v1".as_slice(),
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

pub fn build_attachment_translation_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentTranslationSourcePreparedV1,
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = validate_prepared(&payload)?;
    build_result(
        command_message_id,
        request_id,
        id16(&payload.translation_run_id)?,
        ATTACHMENT_TRANSLATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        attachment_translation_source_prepared_message_id_v1(request_id),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_attachment_translation_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: AttachmentTranslationSourceRejectedV1,
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let request_id = id16(&payload.request_id)?;
    id16(&payload.source_extraction_run_id)?;
    if payload.code == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        command_message_id,
        request_id,
        id16(&payload.translation_run_id)?,
        ATTACHMENT_TRANSLATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        attachment_translation_source_rejected_message_id_v1(request_id),
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
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTranslationSourceEnvelopeBuildErrorV1> {
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
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: ATTACHMENT_TRANSLATION_INGRESS_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: ATTACHMENT_TRANSLATION_INGRESS_CONTRACT_MAJOR_V1,
            revision: ATTACHMENT_TRANSLATION_INGRESS_CONTRACT_REVISION_V1,
            schema_sha256: ATTACHMENT_TRANSLATION_INGRESS_SCHEMA_SHA256.to_vec(),
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
        .map_err(|_| AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_request(
    payload: &RequestAttachmentTranslationSourceV1,
) -> Result<[u8; 16], AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    let translation_run_id = id16(&payload.translation_run_id)?;
    let source_extraction_run_id = id16(&payload.source_extraction_run_id)?;
    if payload.expected_source_revision == 0
        || request_id
            != attachment_translation_source_request_id_v1(
                translation_run_id,
                source_extraction_run_id,
                payload.expected_source_revision,
            )
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_prepared(
    payload: &AttachmentTranslationSourcePreparedV1,
) -> Result<[u8; 16], AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    let request_id = id16(&payload.request_id)?;
    id16(&payload.translation_run_id)?;
    id16(&payload.source_extraction_run_id)?;
    id16(&payload.source_reference_id)?;
    sha256(&payload.receipt_sha256)?;
    if payload.source_revision == 0
        || !(1..=ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&payload.declared_size)
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len() > ATTACHMENT_TRANSLATION_MAX_PROOF_BYTES_V1
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(request_id)
}

fn validate_context(
    context: &AttachmentTranslationSourceEnvelopeContextV1,
) -> Result<(), AttachmentTranslationSourceEnvelopeBuildErrorV1> {
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
        Err(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload)?;
    valid_id(value)?;
    Ok(value)
}

fn sha256(bytes: &[u8]) -> Result<[u8; 32], AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    let value: [u8; 32] = bytes
        .try_into()
        .map_err(|_| AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_id(value: [u8; 16]) -> Result<(), AttachmentTranslationSourceEnvelopeBuildErrorV1> {
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(())
        .ok_or(AttachmentTranslationSourceEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn timestamp(context: &AttachmentTranslationSourceEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn outbox_error(_: OutboxRecordError) -> AttachmentTranslationSourceEnvelopeBuildErrorV1 {
    AttachmentTranslationSourceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    #[test]
    fn command_is_exact_bounded_and_private() {
        let translation_run_id = [2; 16];
        let source_extraction_run_id = [3; 16];
        let payload = RequestAttachmentTranslationSourceV1 {
            request_id: attachment_translation_source_request_id_v1(
                translation_run_id,
                source_extraction_run_id,
                7,
            )
            .to_vec(),
            translation_run_id: translation_run_id.to_vec(),
            source_extraction_run_id: source_extraction_run_id.to_vec(),
            expected_source_revision: 7,
            logical_owner_id: "owner-1".to_owned(),
        };
        let record = build_request_attachment_translation_source_outbox_record_v1(
            payload,
            1_800_000_030,
            &AttachmentTranslationSourceEnvelopeContextV1 {
                module_id: "makosh-attachment-translation-runtime".to_owned(),
                runtime_instance_id: "translation-1".to_owned(),
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
            ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1
        );
    }
}
