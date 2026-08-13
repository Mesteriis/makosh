use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1,
    v1::{
        CancelOneShotScheduleV1, EnsureOneShotScheduleV1, JobKindV1,
        SchedulerScheduleControlCommandV1, scheduler_schedule_control_command_v1::Operation,
    },
    validate_scheduler_schedule_control_command_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use makosh_calendar_api::{CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1};

use crate::admission::{scheduler_job_contract_v1, scheduler_schedule_control_contract_v1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSchedulerEnvelopeContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub recorded_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarSchedulerEnvelopeErrorV1 {
    InvalidInput,
    InvalidEnvelope,
}

pub fn build_ensure_reminder_schedule_v1(
    operation_id: [u8; 16],
    reminder_id: [u8; 16],
    due_at_unix_millis: i64,
    context: &CalendarSchedulerEnvelopeContextV1,
) -> Result<OutboxRecordV1, CalendarSchedulerEnvelopeErrorV1> {
    build_schedule_control(
        operation_id,
        reminder_id,
        Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
            schedule_id: reminder_id.to_vec(),
            schedule_revision: 1,
            job_kind: Some(job_kind()),
            job_contract_revision: u64::from(scheduler_job_contract_v1().revision),
            job_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
            scope_id: encode_id(reminder_id),
            concurrency_key: encode_id(reminder_id),
            due_at_unix_millis,
            deadline_millis: 60_000,
            max_attempts: 3,
            retry_base_backoff_millis: 1_000,
        }),
        context,
    )
}

pub fn build_cancel_reminder_schedule_v1(
    operation_id: [u8; 16],
    reminder_id: [u8; 16],
    context: &CalendarSchedulerEnvelopeContextV1,
) -> Result<OutboxRecordV1, CalendarSchedulerEnvelopeErrorV1> {
    build_schedule_control(
        operation_id,
        reminder_id,
        Operation::CancelOneShot(CancelOneShotScheduleV1 {
            schedule_id: reminder_id.to_vec(),
            expected_schedule_revision: 1,
            job_kind: Some(job_kind()),
        }),
        context,
    )
}

fn build_schedule_control(
    operation_id: [u8; 16],
    reminder_id: [u8; 16],
    operation: Operation,
    context: &CalendarSchedulerEnvelopeContextV1,
) -> Result<OutboxRecordV1, CalendarSchedulerEnvelopeErrorV1> {
    validate_context(context)?;
    if !nonzero(&operation_id) || !nonzero(&reminder_id) {
        return Err(CalendarSchedulerEnvelopeErrorV1::InvalidInput);
    }
    let payload = SchedulerScheduleControlCommandV1 {
        operation_id: operation_id.to_vec(),
        operation: Some(operation),
    };
    validate_scheduler_schedule_control_command_v1(&payload)
        .map_err(|_| CalendarSchedulerEnvelopeErrorV1::InvalidInput)?;
    let contract = scheduler_schedule_control_contract_v1();
    let recorded_at = timestamp(context.recorded_at_unix_millis)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: calendar_schedule_control_message_id_v1(operation_id, reminder_id).to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: CALENDAR_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(recorded_at),
        partition_key: owner_partition(&context.logical_owner_id).to_vec(),
        causation_message_id: operation_id.to_vec(),
        correlation_id: reminder_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: CALENDAR_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::GrantEpoch as i32,
            scope_id: CALENDAR_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.grant_epoch,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: "scheduler_schedule_control".to_owned(),
            idempotency_key: Sha256::digest(payload.encode_to_vec()).to_vec(),
            deadline: Some(Timestamp {
                seconds: recorded_at.seconds + 300,
                nanos: recorded_at.nanos,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CalendarSchedulerEnvelopeErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

#[must_use]
pub fn calendar_schedule_control_message_id_v1(
    operation_id: [u8; 16],
    reminder_id: [u8; 16],
) -> [u8; 16] {
    digest16(
        b"makosh.calendar.schedule-control-message.v1\0",
        &operation_id,
        &reminder_id,
    )
}

fn job_kind() -> JobKindV1 {
    JobKindV1 {
        owner: CALENDAR_OWNER_ID_V1.to_owned(),
        name: "reminder_due".to_owned(),
        major: 1,
    }
}

fn validate_context(
    context: &CalendarSchedulerEnvelopeContextV1,
) -> Result<(), CalendarSchedulerEnvelopeErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.logical_owner_id.len() > 128
        || !nonzero(&context.runtime_instance_id)
        || context.runtime_generation == 0
        || context.grant_epoch == 0
        || context.recorded_at_unix_millis <= 0
    {
        return Err(CalendarSchedulerEnvelopeErrorV1::InvalidInput);
    }
    Ok(())
}

fn timestamp(unix_millis: i64) -> Result<Timestamp, CalendarSchedulerEnvelopeErrorV1> {
    if unix_millis <= 0 {
        return Err(CalendarSchedulerEnvelopeErrorV1::InvalidInput);
    }
    Ok(Timestamp {
        seconds: unix_millis / 1_000,
        nanos: ((unix_millis % 1_000) * 1_000_000) as i32,
    })
}

fn owner_partition(logical_owner_id: &str) -> [u8; 16] {
    digest16(
        b"makosh.calendar.owner-partition.v1\0",
        logical_owner_id.as_bytes(),
        b"calendar",
    )
}

fn digest16(domain: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((left.len() as u64).to_be_bytes());
    hash.update(left);
    hash.update((right.len() as u64).to_be_bytes());
    hash.update(right);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn encode_id(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(32);
    for byte in value {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn outbox_error(_: OutboxRecordError) -> CalendarSchedulerEnvelopeErrorV1 {
    CalendarSchedulerEnvelopeErrorV1::InvalidEnvelope
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> CalendarSchedulerEnvelopeContextV1 {
        CalendarSchedulerEnvelopeContextV1 {
            logical_owner_id: "owner-1".to_owned(),
            runtime_instance_id: [7; 16],
            runtime_generation: 3,
            grant_epoch: 4,
            recorded_at_unix_millis: 1_000,
        }
    }

    #[test]
    fn ensure_and_cancel_are_exact_scheduler_commands() {
        let ensure = build_ensure_reminder_schedule_v1([1; 16], [2; 16], 20_000, &context())
            .expect("ensure");
        let cancel =
            build_cancel_reminder_schedule_v1([3; 16], [2; 16], &context()).expect("cancel");
        for record in [ensure, cancel] {
            let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode");
            assert_eq!(
                envelope.source_fence.expect("fence").kind,
                FenceKindV1::GrantEpoch as i32
            );
            assert_eq!(envelope.contract.expect("contract").owner, "scheduler");
        }
    }
}
