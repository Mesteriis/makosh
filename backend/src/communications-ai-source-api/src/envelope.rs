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
    COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_MAX_BYTES_V1,
    COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_SOURCE_MAX_BYTES_V1,
    COMMUNICATION_EXPLANATION_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_EXPLANATION_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_EXPLANATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1, COMMUNICATION_REPLY_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_REPLY_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_REPLY_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_REPLY_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATION_SUMMARY_SOURCE_MAX_BYTES_V1, COMMUNICATION_SUMMARY_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_SUMMARY_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_SUMMARY_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_SUMMARY_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATION_TRANSLATION_SOURCE_MAX_BYTES_V1,
    COMMUNICATION_TRANSLATION_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATION_TRANSLATION_SOURCE_PREPARE_CONTRACT_NAME_V1,
    COMMUNICATION_TRANSLATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
    COMMUNICATION_TRANSLATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
    COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1, COMMUNICATIONS_AI_SOURCE_CONTRACT_MAJOR_V1,
    COMMUNICATIONS_AI_SOURCE_CONTRACT_REVISION_V1, COMMUNICATIONS_AI_SOURCE_OWNER_V1,
    COMMUNICATIONS_AI_SOURCE_SCHEMA_SHA256,
    COMMUNICATIONS_CALL_TRANSCRIPTION_SOURCE_CAPABILITY_ID_V1,
    COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID_V1,
    COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID_V1,
    COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID_V1,
    wire::{
        CallTranscriptionSourceContentReceiptV1, CallTranscriptionSourcePreparedV1,
        CallTranscriptionSourceRejectedV1, CommunicationExplanationSourceContentReceiptV1,
        CommunicationExplanationSourcePreparedV1, CommunicationExplanationSourceRejectedV1,
        CommunicationReplySourceContentReceiptV1, CommunicationReplySourcePreparedV1,
        CommunicationReplySourceRejectedV1, CommunicationSummarySourceContentReceiptV1,
        CommunicationSummarySourcePreparedV1, CommunicationSummarySourceRejectedV1,
        CommunicationTranslationSourceContentReceiptV1, CommunicationTranslationSourcePreparedV1,
        CommunicationTranslationSourceRejectedV1, PrepareCallTranscriptionSourceCommandV1,
        PrepareCommunicationExplanationSourceCommandV1, PrepareCommunicationReplySourceCommandV1,
        PrepareCommunicationSummarySourceCommandV1, PrepareCommunicationTranslationSourceCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationReplySourceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationReplySourceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub type CommunicationSummarySourceEnvelopeContextV1 = CommunicationReplySourceEnvelopeContextV1;
pub type CommunicationSummarySourceEnvelopeBuildErrorV1 =
    CommunicationReplySourceEnvelopeBuildErrorV1;
pub type CommunicationCallTranscriptionSourceEnvelopeContextV1 =
    CommunicationReplySourceEnvelopeContextV1;
pub type CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1 =
    CommunicationReplySourceEnvelopeBuildErrorV1;
pub type CommunicationTranslationSourceEnvelopeContextV1 =
    CommunicationReplySourceEnvelopeContextV1;
pub type CommunicationTranslationSourceEnvelopeBuildErrorV1 =
    CommunicationReplySourceEnvelopeBuildErrorV1;
pub type CommunicationExplanationSourceEnvelopeContextV1 =
    CommunicationReplySourceEnvelopeContextV1;
pub type CommunicationExplanationSourceEnvelopeBuildErrorV1 =
    CommunicationReplySourceEnvelopeBuildErrorV1;

pub fn build_communication_reply_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationReplySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&run_id)
        || !valid_id(&source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCommunicationReplySourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_REPLY_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-ai-reply-source-prepare-v1".as_slice(),
                    &run_id,
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
        payload,
        context,
    )
}

pub fn build_communication_reply_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationReplySourcePreparedV1,
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationReplySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        command_message_id,
        run_id,
        "prepared",
        COMMUNICATION_REPLY_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_reply_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationReplySourceRejectedV1,
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationReplySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        command_message_id,
        run_id,
        "rejected",
        COMMUNICATION_REPLY_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_call_transcription_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationCallTranscriptionSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&run_id)
        || !valid_id(&source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCallTranscriptionSourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_CALL_TRANSCRIPTION_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-ai-call-transcription-source-prepare-v1".as_slice(),
                    &run_id,
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
        payload,
        context,
    )
}

pub fn build_communication_call_transcription_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CallTranscriptionSourcePreparedV1,
    context: &CommunicationCallTranscriptionSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_call_transcription_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_call_transcription_result(
        command_message_id,
        run_id,
        "prepared",
        COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_call_transcription_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CallTranscriptionSourceRejectedV1,
    context: &CommunicationCallTranscriptionSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_call_transcription_result(
        command_message_id,
        run_id,
        "rejected",
        COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_summary_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationSummarySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationSummarySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&run_id)
        || !valid_id(&source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCommunicationSummarySourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_SUMMARY_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-ai-summary-source-prepare-v1".as_slice(),
                    &run_id,
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
        payload,
        context,
    )
}

pub fn build_communication_summary_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationSummarySourcePreparedV1,
    context: &CommunicationSummarySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationSummarySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_summary_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_summary_result(
        command_message_id,
        run_id,
        "prepared",
        COMMUNICATION_SUMMARY_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_summary_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationSummarySourceRejectedV1,
    context: &CommunicationSummarySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationSummarySourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_summary_result(
        command_message_id,
        run_id,
        "rejected",
        COMMUNICATION_SUMMARY_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_translation_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&run_id)
        || !valid_id(&source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCommunicationTranslationSourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_TRANSLATION_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-ai-translation-source-prepare-v1".as_slice(),
                    &run_id,
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
        payload,
        context,
    )
}

pub fn build_communication_translation_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationTranslationSourcePreparedV1,
    context: &CommunicationTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_translation_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_translation_result(
        command_message_id,
        run_id,
        "prepared",
        COMMUNICATION_TRANSLATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_translation_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationTranslationSourceRejectedV1,
    context: &CommunicationTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationTranslationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_translation_result(
        command_message_id,
        run_id,
        "rejected",
        COMMUNICATION_TRANSLATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_explanation_source_prepare_outbox_record_v1(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CommunicationExplanationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationExplanationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if !valid_id(&run_id)
        || !valid_id(&source_message_id)
        || expected_source_revision == 0
        || !valid_logical_owner_id(logical_owner_id)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCommunicationExplanationSourceCommandV1 {
        run_id: run_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        expected_source_revision,
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        run_id,
        &run_id,
        &[],
        COMMUNICATION_EXPLANATION_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: run_id.to_vec(),
            target_capability: COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-ai-explanation-source-prepare-v1".as_slice(),
                    &run_id,
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
        payload,
        context,
    )
}

pub fn build_communication_explanation_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationExplanationSourcePreparedV1,
    context: &CommunicationExplanationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationExplanationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = validate_explanation_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_explanation_result(
        command_message_id,
        run_id,
        "prepared",
        COMMUNICATION_EXPLANATION_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_communication_explanation_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CommunicationExplanationSourceRejectedV1,
    context: &CommunicationExplanationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationExplanationSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_explanation_result(
        command_message_id,
        run_id,
        "rejected",
        COMMUNICATION_EXPLANATION_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

fn build_explanation_result(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    label: &str,
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &CommunicationExplanationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationExplanationSourceEnvelopeBuildErrorV1> {
    build_envelope(
        explanation_result_message_id(label.as_bytes(), &run_id),
        &run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: run_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build_translation_result(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    label: &str,
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &CommunicationTranslationSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationTranslationSourceEnvelopeBuildErrorV1> {
    build_envelope(
        translation_result_message_id(label.as_bytes(), &run_id),
        &run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: run_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build_call_transcription_result(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    label: &str,
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &CommunicationCallTranscriptionSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1> {
    build_envelope(
        call_transcription_result_message_id(label.as_bytes(), &run_id),
        &run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: run_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build_summary_result(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    label: &str,
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &CommunicationSummarySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationSummarySourceEnvelopeBuildErrorV1> {
    build_envelope(
        summary_result_message_id(label.as_bytes(), &run_id),
        &run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: run_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build_result(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    label: &str,
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationReplySourceEnvelopeBuildErrorV1> {
    build_envelope(
        result_message_id(label.as_bytes(), &run_id),
        &run_id,
        &command_message_id,
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: run_id.to_vec(),
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
    partition_key: &[u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CommunicationReplySourceEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: COMMUNICATIONS_AI_SOURCE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: COMMUNICATIONS_AI_SOURCE_CONTRACT_MAJOR_V1,
            revision: COMMUNICATIONS_AI_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATIONS_AI_SOURCE_SCHEMA_SHA256.to_vec(),
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
        .map_err(|_| CommunicationReplySourceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &CommunicationReplySourceEnvelopeContextV1,
) -> Result<(), CommunicationReplySourceEnvelopeBuildErrorV1> {
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
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_prepared_payload(
    payload: &CommunicationReplySourcePreparedV1,
) -> Result<[u8; 16], CommunicationReplySourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    id16(&payload.source_message_id)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_source_receipt(receipt))
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn validate_summary_prepared_payload(
    payload: &CommunicationSummarySourcePreparedV1,
) -> Result<[u8; 16], CommunicationSummarySourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    id16(&payload.source_message_id)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_summary_source_receipt(receipt))
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn validate_translation_prepared_payload(
    payload: &CommunicationTranslationSourcePreparedV1,
) -> Result<[u8; 16], CommunicationTranslationSourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    id16(&payload.source_message_id)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_translation_source_receipt(receipt))
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn validate_call_transcription_prepared_payload(
    payload: &CallTranscriptionSourcePreparedV1,
) -> Result<[u8; 16], CommunicationCallTranscriptionSourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    id16(&payload.source_message_id)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_call_transcription_source_receipt(receipt))
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn validate_explanation_prepared_payload(
    payload: &CommunicationExplanationSourcePreparedV1,
) -> Result<[u8; 16], CommunicationExplanationSourceEnvelopeBuildErrorV1> {
    let run_id = id16(&payload.run_id)?;
    id16(&payload.source_message_id)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload
            .source_content
            .as_ref()
            .is_none_or(|receipt| !valid_explanation_source_receipt(receipt))
    {
        return Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(run_id)
}

fn valid_explanation_source_receipt(
    receipt: &CommunicationExplanationSourceContentReceiptV1,
) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_EXPLANATION_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_EXPLANATION_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_translation_source_receipt(
    receipt: &CommunicationTranslationSourceContentReceiptV1,
) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_TRANSLATION_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_TRANSLATION_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_summary_source_receipt(receipt: &CommunicationSummarySourceContentReceiptV1) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_SUMMARY_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_SUMMARY_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_call_transcription_source_receipt(
    receipt: &CallTranscriptionSourceContentReceiptV1,
) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_MAX_BYTES_V1)
            .contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_CALL_TRANSCRIPTION_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_source_receipt(receipt: &CommunicationReplySourceContentReceiptV1) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= COMMUNICATION_REPLY_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], CommunicationReplySourceEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload)
}

fn timestamp(context: &CommunicationReplySourceEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-reply-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn summary_result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-summary-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn call_transcription_result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-call-transcription-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn translation_result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-translation-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn explanation_result_message_id(label: &[u8], run_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-explanation-source-result-v1");
    hasher.update(label);
    hasher.update(run_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> CommunicationReplySourceEnvelopeBuildErrorV1 {
    CommunicationReplySourceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context() -> CommunicationReplySourceEnvelopeContextV1 {
        CommunicationReplySourceEnvelopeContextV1 {
            module_id: "makosh-communication-reply-suggestion-runtime".to_owned(),
            runtime_instance_id: "reply-source-runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    #[test]
    fn command_is_bounded_and_has_exact_capability() {
        let record = build_communication_reply_source_prepare_outbox_record_v1(
            [1; 16],
            [2; 16],
            9,
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("valid command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Semantics::Command(command) = envelope.semantics.expect("semantics") else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID_V1
        );
        let payload = PrepareCommunicationReplySourceCommandV1::decode(envelope.payload.as_slice())
            .expect("payload");
        assert_eq!(payload.expected_source_revision, 9);
        assert_eq!(payload.logical_owner_id, "owner-1");
    }

    #[test]
    fn prepared_rejects_oversize_or_empty_custody_proof() {
        let payload = CommunicationReplySourcePreparedV1 {
            run_id: vec![1; 16],
            source_message_id: vec![2; 16],
            source_evidence_id: vec![3; 16],
            source_evidence_revision: 9,
            source_content: Some(CommunicationReplySourceContentReceiptV1 {
                reference_id: vec![4; 16],
                declared_bytes: COMMUNICATION_REPLY_SOURCE_MAX_BYTES_V1 + 1,
                sha256: vec![5; 32],
                custody_transfer_source_proof: Vec::new(),
            }),
            logical_owner_id: "owner-1".to_owned(),
        };
        assert_eq!(
            build_communication_reply_source_prepared_outbox_record_v1(
                [6; 16],
                payload,
                &context()
            ),
            Err(CommunicationReplySourceEnvelopeBuildErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn summary_command_uses_distinct_contract_and_capability() {
        let record = build_communication_summary_source_prepare_outbox_record_v1(
            [7; 16],
            [8; 16],
            3,
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("summary command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.contract.expect("contract").name,
            COMMUNICATION_SUMMARY_SOURCE_PREPARE_CONTRACT_NAME_V1
        );
        let Semantics::Command(command) = envelope.semantics.expect("semantics") else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID_V1
        );
    }

    #[test]
    fn translation_command_uses_distinct_contract_and_capability() {
        let record = build_communication_translation_source_prepare_outbox_record_v1(
            [9; 16],
            [10; 16],
            4,
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("translation command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.contract.expect("contract").name,
            COMMUNICATION_TRANSLATION_SOURCE_PREPARE_CONTRACT_NAME_V1
        );
        let Semantics::Command(command) = envelope.semantics.expect("semantics") else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID_V1
        );
    }

    #[test]
    fn explanation_command_uses_distinct_contract_and_capability() {
        let record = build_communication_explanation_source_prepare_outbox_record_v1(
            [11; 16],
            [12; 16],
            5,
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("explanation command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.contract.expect("contract").name,
            COMMUNICATION_EXPLANATION_SOURCE_PREPARE_CONTRACT_NAME_V1
        );
        let Semantics::Command(command) = envelope.semantics.expect("semantics") else {
            panic!("command");
        };
        assert_eq!(
            command.target_capability,
            COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID_V1
        );
    }
}
