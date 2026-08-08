//! Durable Communications replay command consumer.

use makosh_communications_retained_evidence_replay_contract::{
    COMMUNICATIONS_REPLAY_CAPABILITY_ID_V1, COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1,
    CommunicationsReplayResultEnvelopeContextV1, build_communications_replay_result_outbox_v1,
    communications_replay_command_contract_reference_v1, validate_communications_replay_command_v1,
    wire::{
        ReplayCommunicationsEvidenceCommandV1, ReplayCommunicationsEvidenceFailureV1,
        ReplayCommunicationsEvidenceOutcomeV1, ReplayCommunicationsEvidenceResultV1,
    },
};
use makosh_communications_retained_evidence_replay_persistence::{
    CommunicationsReplayCommandAdmissionV1, CommunicationsReplayCommandInboxOutcomeV1,
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsReplayErrorV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePublishPermitV1, RuntimeSubscribePermitV1,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::retained_evidence_replay::{
    CommunicationsReplayExecutionContextV1, CommunicationsRetainedEvidenceReplayErrorV1,
    replay_retained_communications_evidence_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsReplayConsumerContextV1 {
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
pub struct DecodedCommunicationsReplayCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command: ReplayCommunicationsEvidenceCommandV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsReplayCommandDecodeErrorV1 {
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongAudience,
    InvalidPayload,
    OwnerMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsReplayCommandConsumeOutcomeV1 {
    Completed,
    DuplicateCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsReplayCommandConsumeErrorV1 {
    EventUnavailable,
    Decode(CommunicationsReplayCommandDecodeErrorV1),
    Persistence(RetainedCommunicationsReplayErrorV1),
    ResultEnvelope,
    ReplayRetryable,
}

pub async fn consume_next_communications_replay_command_v1(
    persistence: &CommunicationsRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    command_permit: &RuntimeSubscribePermitV1,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    context: &CommunicationsReplayConsumerContextV1,
) -> Result<
    Option<CommunicationsReplayCommandConsumeOutcomeV1>,
    CommunicationsReplayCommandConsumeErrorV1,
> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, command_permit)
        .await
        .map_err(|_| CommunicationsReplayCommandConsumeErrorV1::EventUnavailable)?
    else {
        return Ok(None);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec()).map_err(|_| {
        CommunicationsReplayCommandConsumeErrorV1::Decode(
            CommunicationsReplayCommandDecodeErrorV1::InvalidEnvelope,
        )
    })?;
    let outcome = accept_communications_replay_command_v1(
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
        .map_err(|_| CommunicationsReplayCommandConsumeErrorV1::EventUnavailable)?;
    Ok(Some(outcome))
}

pub async fn accept_communications_replay_command_v1(
    persistence: &CommunicationsRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    record: &OutboxRecordV1,
    context: &CommunicationsReplayConsumerContextV1,
) -> Result<CommunicationsReplayCommandConsumeOutcomeV1, CommunicationsReplayCommandConsumeErrorV1>
{
    let decoded = decode_communications_replay_command_envelope_v1(record)
        .map_err(CommunicationsReplayCommandConsumeErrorV1::Decode)?;
    let owner_mismatch = decoded.command.logical_owner_id != context.logical_owner_id;
    let operation_id = id16(&decoded.command.operation_id)
        .map_err(CommunicationsReplayCommandConsumeErrorV1::Decode)?;
    let admission = CommunicationsReplayCommandAdmissionV1 {
        command_message_id: decoded.command_message_id,
        command_envelope_sha256: decoded.command_envelope_sha256,
        operation_id,
        logical_owner_id: context.logical_owner_id.clone(),
    };
    let inbox = persistence
        .accept_replay_command(&admission, context.completed_at_unix_seconds)
        .await
        .map_err(CommunicationsReplayCommandConsumeErrorV1::Persistence)?;
    if inbox == CommunicationsReplayCommandInboxOutcomeV1::DuplicateCompleted {
        return Ok(CommunicationsReplayCommandConsumeOutcomeV1::DuplicateCompleted);
    }
    let result = if owner_mismatch {
        owner_mismatch_result(&decoded.command)
    } else {
        match replay_retained_communications_evidence_v1(
            persistence,
            connection,
            original_contract_publish_permit,
            &decoded.command,
            &CommunicationsReplayExecutionContextV1 {
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
                .ok_or(CommunicationsReplayCommandConsumeErrorV1::ReplayRetryable)?,
        }
    };
    let result_record = build_communications_replay_result_outbox_v1(
        decoded.command_message_id,
        result,
        &CommunicationsReplayResultEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            completed_at_unix_seconds: context.completed_at_unix_seconds,
            completed_at_nanos: context.completed_at_nanos,
            execution_attempt: context.execution_attempt,
        },
    )
    .map_err(|_| CommunicationsReplayCommandConsumeErrorV1::ResultEnvelope)?;
    persistence
        .complete_replay_command(
            &admission,
            &result_record,
            context.completed_at_unix_seconds,
        )
        .await
        .map_err(CommunicationsReplayCommandConsumeErrorV1::Persistence)?;
    Ok(CommunicationsReplayCommandConsumeOutcomeV1::Completed)
}

pub fn decode_communications_replay_command_v1(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<DecodedCommunicationsReplayCommandV1, CommunicationsReplayCommandDecodeErrorV1> {
    let decoded = decode_communications_replay_command_envelope_v1(record)?;
    if decoded.command.logical_owner_id != expected_logical_owner_id {
        return Err(CommunicationsReplayCommandDecodeErrorV1::OwnerMismatch);
    }
    Ok(decoded)
}

fn decode_communications_replay_command_envelope_v1(
    record: &OutboxRecordV1,
) -> Result<DecodedCommunicationsReplayCommandV1, CommunicationsReplayCommandDecodeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsReplayCommandDecodeErrorV1::InvalidEnvelope)?;
    let expected_contract = communications_replay_command_contract_reference_v1();
    if !exact_contract(envelope.contract.as_ref(), &expected_contract) {
        return Err(CommunicationsReplayCommandDecodeErrorV1::WrongContract);
    }
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsReplayCommandDecodeErrorV1::WrongContract);
    };
    if metadata.target_capability != COMMUNICATIONS_REPLAY_CAPABILITY_ID_V1 {
        return Err(CommunicationsReplayCommandDecodeErrorV1::WrongAudience);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(CommunicationsReplayCommandDecodeErrorV1::WrongSource)?;
    if source.module_id != COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1
        || source.runtime_generation == 0
        || envelope.source_fence.as_ref().is_none_or(|fence| {
            fence.scope_id != COMMUNICATIONS_REPLAY_SOURCE_MODULE_ID_V1.as_bytes()
                || fence.epoch != source.runtime_generation
        })
    {
        return Err(CommunicationsReplayCommandDecodeErrorV1::WrongSource);
    }
    let command = ReplayCommunicationsEvidenceCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsReplayCommandDecodeErrorV1::InvalidPayload)?;
    validate_communications_replay_command_v1(&command)
        .map_err(|_| CommunicationsReplayCommandDecodeErrorV1::InvalidPayload)?;
    let operation_id = id16(&command.operation_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    if metadata.command_id.as_slice() != operation_id
        || envelope.partition_key.as_slice() != operation_id
        || envelope.correlation_id.as_slice() != operation_id
    {
        return Err(CommunicationsReplayCommandDecodeErrorV1::WrongContract);
    }
    Ok(DecodedCommunicationsReplayCommandV1 {
        command_message_id,
        command_envelope_sha256: *record.envelope_sha256(),
        command,
    })
}

fn owner_mismatch_result(
    command: &ReplayCommunicationsEvidenceCommandV1,
) -> ReplayCommunicationsEvidenceResultV1 {
    ReplayCommunicationsEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: ReplayCommunicationsEvidenceOutcomeV1::Rejected as i32,
        original_message_ids: Vec::new(),
        failure: ReplayCommunicationsEvidenceFailureV1::OwnerMismatch as i32,
    }
}

fn terminal_result(
    command: &ReplayCommunicationsEvidenceCommandV1,
    error: CommunicationsRetainedEvidenceReplayErrorV1,
) -> Option<ReplayCommunicationsEvidenceResultV1> {
    let (outcome, failure) = match error {
        CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand => (
            ReplayCommunicationsEvidenceOutcomeV1::Rejected,
            ReplayCommunicationsEvidenceFailureV1::WrongContract,
        ),
        CommunicationsRetainedEvidenceReplayErrorV1::Persistence(error) => match error {
            RetainedCommunicationsReplayErrorV1::NotFound => (
                ReplayCommunicationsEvidenceOutcomeV1::Unavailable,
                ReplayCommunicationsEvidenceFailureV1::NotFound,
            ),
            RetainedCommunicationsReplayErrorV1::HashMismatch => (
                ReplayCommunicationsEvidenceOutcomeV1::Unavailable,
                ReplayCommunicationsEvidenceFailureV1::HashMismatch,
            ),
            RetainedCommunicationsReplayErrorV1::WrongContract
            | RetainedCommunicationsReplayErrorV1::InvalidInput
            | RetainedCommunicationsReplayErrorV1::InvalidRow => (
                ReplayCommunicationsEvidenceOutcomeV1::Unavailable,
                ReplayCommunicationsEvidenceFailureV1::WrongContract,
            ),
            RetainedCommunicationsReplayErrorV1::Conflict
            | RetainedCommunicationsReplayErrorV1::StorageUnavailable => return None,
        },
        CommunicationsRetainedEvidenceReplayErrorV1::PublishUnavailable => return None,
    };
    Some(ReplayCommunicationsEvidenceResultV1 {
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

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsReplayCommandDecodeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsReplayCommandDecodeErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_communications_retained_evidence_replay_contract::{
        CommunicationsReplayCommandEnvelopeContextV1, build_communications_replay_command_outbox_v1,
    };
    use makosh_events_protocol::{
        delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
        validation::envelope::decode_envelope_v1,
    };

    use super::*;

    fn command() -> ReplayCommunicationsEvidenceCommandV1 {
        ReplayCommunicationsEvidenceCommandV1 {
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
            CommunicationsRetainedEvidenceReplayErrorV1::Persistence(
                RetainedCommunicationsReplayErrorV1::NotFound,
            ),
        )
        .expect("terminal");
        assert_eq!(
            not_found.failure,
            ReplayCommunicationsEvidenceFailureV1::NotFound as i32
        );
        assert!(
            terminal_result(
                &command(),
                CommunicationsRetainedEvidenceReplayErrorV1::PublishUnavailable,
            )
            .is_none()
        );
        let wrong_owner = owner_mismatch_result(&command());
        assert_eq!(
            wrong_owner.failure,
            ReplayCommunicationsEvidenceFailureV1::OwnerMismatch as i32
        );
        assert_eq!(
            wrong_owner.outcome,
            ReplayCommunicationsEvidenceOutcomeV1::Rejected as i32
        );
    }

    #[test]
    fn decoder_accepts_only_exact_workflow_source_capability_and_owner() {
        let record = build_communications_replay_command_outbox_v1(
            command(),
            &CommunicationsReplayCommandEnvelopeContextV1 {
                runtime_instance_id: "replay-runtime-1".to_owned(),
                runtime_generation: 11,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 0,
                deadline_unix_seconds: 1_700_000_030,
                logical_attempt: 1,
            },
        )
        .expect("command envelope");
        let decoded =
            decode_communications_replay_command_v1(&record, "owner-1").expect("exact command");
        assert_eq!(decoded.command.operation_id, vec![1; 16]);
        assert_eq!(
            decode_communications_replay_command_v1(&record, "owner-2"),
            Err(CommunicationsReplayCommandDecodeErrorV1::OwnerMismatch)
        );

        let mut wrong_source = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        wrong_source.source.as_mut().expect("source").module_id = "wrong-workflow".to_owned();
        let wrong_source = OutboxRecordV1::accept(wrong_source.encode_to_vec()).expect("record");
        assert_eq!(
            decode_communications_replay_command_v1(&wrong_source, "owner-1"),
            Err(CommunicationsReplayCommandDecodeErrorV1::WrongSource)
        );

        let mut wrong_audience = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(metadata)) = wrong_audience.semantics.as_mut() else {
            panic!("command");
        };
        metadata.target_capability = "wrong.capability".to_owned();
        let wrong_audience =
            OutboxRecordV1::accept(wrong_audience.encode_to_vec()).expect("record");
        assert_eq!(
            decode_communications_replay_command_v1(&wrong_audience, "owner-1"),
            Err(CommunicationsReplayCommandDecodeErrorV1::WrongAudience)
        );
    }
}
