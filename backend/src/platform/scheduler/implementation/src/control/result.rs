use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1, ResultMetadataV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_scheduler_protocol::{
    v1::{SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1},
    validate_scheduler_schedule_control_result_v1,
};
use prost::Message;
use prost_types::Timestamp;

use crate::SchedulerDispatchIdentityV1;

use super::SchedulerScheduleControlContractV1;

pub fn build_schedule_control_result_envelope_v1(
    command: &OutboxRecordV1,
    payload: SchedulerScheduleControlResultV1,
    result_message_id: [u8; 16],
    completed_at: UtcMillisV1,
    source: &SchedulerDispatchIdentityV1,
    contract: &SchedulerScheduleControlContractV1,
) -> Result<DurableEnvelopeV1, SchedulerScheduleControlResultBuildErrorV1> {
    validate_scheduler_schedule_control_result_v1(&payload)
        .map_err(|_| SchedulerScheduleControlResultBuildErrorV1::InvalidPayload)?;
    if !result_message_id.iter().any(|byte| *byte != 0) {
        return Err(SchedulerScheduleControlResultBuildErrorV1::InvalidMessageId);
    }
    let command_envelope = decode_envelope_v1(command.exact_bytes())
        .map_err(|_| SchedulerScheduleControlResultBuildErrorV1::InvalidCommand)?;
    let Some(Semantics::Command(command_metadata)) = command_envelope.semantics.as_ref() else {
        return Err(SchedulerScheduleControlResultBuildErrorV1::InvalidCommand);
    };
    let command_id = command_metadata.command_id.clone();
    let timestamp = timestamp(completed_at)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: result_message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: "scheduler".to_owned(),
            name: "schedule_control".to_owned(),
            major: 1,
            revision: contract.revision(),
            schema_sha256: contract.schema_sha256().to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: source.runtime_id().to_owned(),
            runtime_instance_id: source.runtime_instance_id().to_vec(),
            runtime_generation: source.runtime_generation(),
        }),
        recorded_at: Some(timestamp),
        partition_key: command_envelope.partition_key,
        causation_message_id: command.message_id().to_vec(),
        correlation_id: command_envelope.correlation_id,
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System.into(),
            actor_id: source.runtime_id().as_bytes().to_vec(),
        }),
        trace: command_envelope.trace,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease.into(),
            scope_id: source.runtime_id().as_bytes().to_vec(),
            epoch: source.runtime_generation(),
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id,
            command_message_id: command.message_id().to_vec(),
            outcome: outer_outcome(&payload)?.into(),
            completed_at: Some(timestamp),
            execution_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| SchedulerScheduleControlResultBuildErrorV1::InvalidEnvelope)?;
    Ok(envelope)
}

fn outer_outcome(
    payload: &SchedulerScheduleControlResultV1,
) -> Result<ResultOutcomeV1, SchedulerScheduleControlResultBuildErrorV1> {
    match SchedulerScheduleControlOutcomeV1::try_from(payload.outcome)
        .map_err(|_| SchedulerScheduleControlResultBuildErrorV1::InvalidPayload)?
    {
        SchedulerScheduleControlOutcomeV1::Rejected => Ok(ResultOutcomeV1::Rejected),
        SchedulerScheduleControlOutcomeV1::Ensured
        | SchedulerScheduleControlOutcomeV1::Cancelled
        | SchedulerScheduleControlOutcomeV1::TooLate => Ok(ResultOutcomeV1::Succeeded),
        SchedulerScheduleControlOutcomeV1::Unspecified => {
            Err(SchedulerScheduleControlResultBuildErrorV1::InvalidPayload)
        }
    }
}

fn timestamp(value: UtcMillisV1) -> Result<Timestamp, SchedulerScheduleControlResultBuildErrorV1> {
    Ok(Timestamp {
        seconds: value.value().div_euclid(1_000),
        nanos: i32::try_from(value.value().rem_euclid(1_000) * 1_000_000)
            .map_err(|_| SchedulerScheduleControlResultBuildErrorV1::InvalidTimestamp)?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlResultBuildErrorV1 {
    InvalidCommand,
    InvalidPayload,
    InvalidMessageId,
    InvalidTimestamp,
    InvalidEnvelope,
}
