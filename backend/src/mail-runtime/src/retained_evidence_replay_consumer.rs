//! Durable Mail replay command consumer.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_retained_evidence_replay_contract::{
    MAIL_REPLAY_CAPABILITY_ID_V1, MAIL_REPLAY_SOURCE_MODULE_ID_V1,
    MailReplayResultEnvelopeContextV1, build_mail_replay_result_outbox_v1,
    mail_replay_command_contract_reference_v1, validate_mail_replay_command_v1,
    wire::{
        ReplayMailEvidenceCommandV1, ReplayMailEvidenceFailureV1, ReplayMailEvidenceOutcomeV1,
        ReplayMailEvidenceResultV1,
    },
};
use makosh_mail_retained_evidence_replay_persistence::{
    MailReplayCommandAdmissionV1, MailReplayCommandInboxOutcomeV1,
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailReplayErrorV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::retained_evidence_replay::{
    MailReplayExecutionContextV1, MailRetainedEvidenceReplayErrorV1,
    replay_retained_mail_evidence_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailReplayConsumerContextV1 {
    pub logical_owner_id: String,
    pub producer_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub execution_attempt: u32,
    pub completed_at_unix_seconds: i64,
    pub completed_at_nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedMailReplayCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command: ReplayMailEvidenceCommandV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayCommandDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongAudience,
    InvalidPayload,
    OwnerMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayCommandConsumeOutcomeV1 {
    Completed,
    DuplicateCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayCommandConsumeErrorV1 {
    EventUnavailable,
    Decode(MailReplayCommandDecodeErrorV1),
    Persistence(RetainedMailReplayErrorV1),
    ResultEnvelope,
    ReplayRetryable,
}

pub async fn consume_next_mail_replay_command_v1(
    persistence: &MailRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    command_permit: &RuntimeSubscribePermitV1,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    context: &MailReplayConsumerContextV1,
) -> Result<Option<MailReplayCommandConsumeOutcomeV1>, MailReplayCommandConsumeErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, command_permit)
        .await
        .map_err(|_| MailReplayCommandConsumeErrorV1::EventUnavailable)?
    else {
        return Ok(None);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec()).map_err(|_| {
        MailReplayCommandConsumeErrorV1::Decode(MailReplayCommandDecodeErrorV1::InvalidEnvelope)
    })?;
    let outcome = accept_mail_replay_command_v1(
        persistence,
        connection,
        original_contract_publish_permit,
        &record,
        context,
    )
    .await?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailReplayCommandConsumeErrorV1::EventUnavailable)?;
    Ok(Some(outcome))
}

pub async fn accept_mail_replay_command_v1(
    persistence: &MailRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    record: &OutboxRecordV1,
    context: &MailReplayConsumerContextV1,
) -> Result<MailReplayCommandConsumeOutcomeV1, MailReplayCommandConsumeErrorV1> {
    let decoded = decode_mail_replay_command_envelope_v1(record)
        .map_err(MailReplayCommandConsumeErrorV1::Decode)?;
    let owner_mismatch = decoded.command.logical_owner_id != context.logical_owner_id;
    let operation_id =
        id16(&decoded.command.operation_id).map_err(MailReplayCommandConsumeErrorV1::Decode)?;
    let admission = MailReplayCommandAdmissionV1 {
        command_message_id: decoded.command_message_id,
        command_envelope_sha256: decoded.command_envelope_sha256,
        operation_id,
        logical_owner_id: context.logical_owner_id.clone(),
    };
    let inbox = persistence
        .accept_replay_command(&admission, context.completed_at_unix_seconds)
        .await
        .map_err(MailReplayCommandConsumeErrorV1::Persistence)?;
    if inbox == MailReplayCommandInboxOutcomeV1::DuplicateCompleted {
        return Ok(MailReplayCommandConsumeOutcomeV1::DuplicateCompleted);
    }
    let result = if owner_mismatch {
        owner_mismatch_result(&decoded.command)
    } else {
        match replay_retained_mail_evidence_v1(
            persistence,
            connection,
            original_contract_publish_permit,
            &decoded.command,
            &MailReplayExecutionContextV1 {
                registration_id: &context.producer_registration_id,
                runtime_generation: context.runtime_generation,
                grant_epoch: context.grant_epoch,
                logical_attempt: context.execution_attempt,
                recorded_at_unix_seconds: context.completed_at_unix_seconds,
            },
        )
        .await
        {
            Ok(result) => result,
            Err(error) => terminal_result(&decoded.command, error)
                .ok_or(MailReplayCommandConsumeErrorV1::ReplayRetryable)?,
        }
    };
    let result_record = build_mail_replay_result_outbox_v1(
        decoded.command_message_id,
        result,
        &MailReplayResultEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            completed_at_unix_seconds: context.completed_at_unix_seconds,
            completed_at_nanos: context.completed_at_nanos,
            execution_attempt: context.execution_attempt,
        },
    )
    .map_err(|_| MailReplayCommandConsumeErrorV1::ResultEnvelope)?;
    persistence
        .complete_replay_command(
            &admission,
            &result_record,
            context.completed_at_unix_seconds,
        )
        .await
        .map_err(MailReplayCommandConsumeErrorV1::Persistence)?;
    Ok(MailReplayCommandConsumeOutcomeV1::Completed)
}

pub fn decode_mail_replay_command_v1(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<DecodedMailReplayCommandV1, MailReplayCommandDecodeErrorV1> {
    let decoded = decode_mail_replay_command_envelope_v1(record)?;
    if decoded.command.logical_owner_id != expected_logical_owner_id {
        return Err(MailReplayCommandDecodeErrorV1::OwnerMismatch);
    }
    Ok(decoded)
}

fn decode_mail_replay_command_envelope_v1(
    record: &OutboxRecordV1,
) -> Result<DecodedMailReplayCommandV1, MailReplayCommandDecodeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailReplayCommandDecodeErrorV1::InvalidEnvelope)?;
    let expected_contract = mail_replay_command_contract_reference_v1();
    if !exact_contract(envelope.contract.as_ref(), &expected_contract) {
        return Err(MailReplayCommandDecodeErrorV1::WrongContract);
    }
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailReplayCommandDecodeErrorV1::WrongContract);
    };
    if metadata.target_capability != MAIL_REPLAY_CAPABILITY_ID_V1 {
        return Err(MailReplayCommandDecodeErrorV1::WrongAudience);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailReplayCommandDecodeErrorV1::WrongSource)?;
    if source.module_id != MAIL_REPLAY_SOURCE_MODULE_ID_V1
        || source.runtime_generation == 0
        || envelope.source_fence.as_ref().is_none_or(|fence| {
            fence.scope_id != MAIL_REPLAY_SOURCE_MODULE_ID_V1.as_bytes()
                || fence.epoch != source.runtime_generation
        })
    {
        return Err(MailReplayCommandDecodeErrorV1::WrongSource);
    }
    let command = ReplayMailEvidenceCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailReplayCommandDecodeErrorV1::InvalidPayload)?;
    validate_mail_replay_command_v1(&command)
        .map_err(|_| MailReplayCommandDecodeErrorV1::InvalidPayload)?;
    let operation_id = id16(&command.operation_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    if metadata.command_id.as_slice() != operation_id
        || envelope.partition_key.as_slice() != operation_id
        || envelope.correlation_id.as_slice() != operation_id
    {
        return Err(MailReplayCommandDecodeErrorV1::WrongContract);
    }
    Ok(DecodedMailReplayCommandV1 {
        command_message_id,
        command_envelope_sha256: *record.envelope_sha256(),
        command,
    })
}

fn owner_mismatch_result(command: &ReplayMailEvidenceCommandV1) -> ReplayMailEvidenceResultV1 {
    ReplayMailEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: ReplayMailEvidenceOutcomeV1::Rejected as i32,
        original_message_ids: Vec::new(),
        failure: ReplayMailEvidenceFailureV1::OwnerMismatch as i32,
    }
}

fn terminal_result(
    command: &ReplayMailEvidenceCommandV1,
    error: MailRetainedEvidenceReplayErrorV1,
) -> Option<ReplayMailEvidenceResultV1> {
    let (outcome, failure) = match error {
        MailRetainedEvidenceReplayErrorV1::InvalidCommand => (
            ReplayMailEvidenceOutcomeV1::Rejected,
            ReplayMailEvidenceFailureV1::WrongContract,
        ),
        MailRetainedEvidenceReplayErrorV1::Persistence(error) => match error {
            RetainedMailReplayErrorV1::NotFound => (
                ReplayMailEvidenceOutcomeV1::Unavailable,
                ReplayMailEvidenceFailureV1::NotFound,
            ),
            RetainedMailReplayErrorV1::HashMismatch => (
                ReplayMailEvidenceOutcomeV1::Unavailable,
                ReplayMailEvidenceFailureV1::HashMismatch,
            ),
            RetainedMailReplayErrorV1::WrongContract
            | RetainedMailReplayErrorV1::InvalidInput
            | RetainedMailReplayErrorV1::InvalidRow => (
                ReplayMailEvidenceOutcomeV1::Unavailable,
                ReplayMailEvidenceFailureV1::WrongContract,
            ),
            RetainedMailReplayErrorV1::Conflict | RetainedMailReplayErrorV1::StorageUnavailable => {
                return None;
            }
        },
        MailRetainedEvidenceReplayErrorV1::PublishUnavailable => return None,
    };
    Some(ReplayMailEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: outcome as i32,
        original_message_ids: Vec::new(),
        failure: failure as i32,
    })
}

fn exact_contract(
    value: Option<&makosh_events_protocol::v1::ContractRefV1>,
    expected: &ContractReferenceV1,
) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256.as_slice() == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailReplayCommandDecodeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailReplayCommandDecodeErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{
        delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
        validation::envelope::decode_envelope_v1,
    };
    use makosh_mail_retained_evidence_replay_contract::{
        MailReplayCommandEnvelopeContextV1, build_mail_replay_command_outbox_v1,
    };

    use super::*;

    fn command() -> ReplayMailEvidenceCommandV1 {
        ReplayMailEvidenceCommandV1 {
            operation_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: vec![2; 32],
            attachment_anchor_id: vec![3; 16],
        }
    }

    #[test]
    fn deterministic_failures_are_terminal_but_transport_outage_is_retryable() {
        let not_found = terminal_result(
            &command(),
            MailRetainedEvidenceReplayErrorV1::Persistence(RetainedMailReplayErrorV1::NotFound),
        )
        .expect("terminal");
        assert_eq!(
            not_found.failure,
            ReplayMailEvidenceFailureV1::NotFound as i32
        );
        assert!(
            terminal_result(
                &command(),
                MailRetainedEvidenceReplayErrorV1::PublishUnavailable,
            )
            .is_none()
        );
        let wrong_owner = owner_mismatch_result(&command());
        assert_eq!(
            wrong_owner.failure,
            ReplayMailEvidenceFailureV1::OwnerMismatch as i32
        );
        assert_eq!(
            wrong_owner.outcome,
            ReplayMailEvidenceOutcomeV1::Rejected as i32
        );
    }

    #[test]
    fn decoder_accepts_only_exact_workflow_source_capability_and_owner() {
        let record = build_mail_replay_command_outbox_v1(
            command(),
            &MailReplayCommandEnvelopeContextV1 {
                runtime_instance_id: "replay-runtime-1".to_owned(),
                runtime_generation: 11,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 0,
                deadline_unix_seconds: 1_700_000_030,
                logical_attempt: 1,
            },
        )
        .expect("command envelope");
        let decoded = decode_mail_replay_command_v1(&record, "owner-1").expect("exact command");
        assert_eq!(decoded.command.operation_id, vec![1; 16]);
        assert_eq!(
            decode_mail_replay_command_v1(&record, "owner-2"),
            Err(MailReplayCommandDecodeErrorV1::OwnerMismatch)
        );

        let mut wrong_source = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        wrong_source.source.as_mut().expect("source").module_id = "wrong-workflow".to_owned();
        let wrong_source = OutboxRecordV1::accept(wrong_source.encode_to_vec()).expect("record");
        assert_eq!(
            decode_mail_replay_command_v1(&wrong_source, "owner-1"),
            Err(MailReplayCommandDecodeErrorV1::WrongSource)
        );

        let mut wrong_audience = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(metadata)) = wrong_audience.semantics.as_mut() else {
            panic!("command");
        };
        metadata.target_capability = "wrong.capability".to_owned();
        let wrong_audience =
            OutboxRecordV1::accept(wrong_audience.encode_to_vec()).expect("record");
        assert_eq!(
            decode_mail_replay_command_v1(&wrong_audience, "owner-1"),
            Err(MailReplayCommandDecodeErrorV1::WrongAudience)
        );
    }
}
