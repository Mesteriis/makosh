//! Creates the exact owner-neutral command envelope for one fenced Scheduler fire.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_scheduler_protocol::v1::JobTriggerKindV1;
use makosh_scheduler_protocol::{
    ScheduleRunLeaseV1, ScheduleSpecV1, build_scheduled_job_command_v1,
};
use prost::Message;
use prost_types::Timestamp;

use super::SchedulerDispatchIdentityV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerJobEnvelopeBuildErrorV1 {
    InvalidDispatchId,
    InvalidFireKey,
    InvalidCommand,
    InvalidTimestamp,
    InvalidEnvelope,
}

pub fn build_scheduled_job_envelope_v1(
    schedule: &ScheduleSpecV1,
    lease: &ScheduleRunLeaseV1,
    scheduled_for: UtcMillisV1,
    trigger: JobTriggerKindV1,
    dispatch_message_id: [u8; 16],
    fire_key: [u8; 32],
    source: &SchedulerDispatchIdentityV1,
) -> Result<DurableEnvelopeV1, SchedulerJobEnvelopeBuildErrorV1> {
    valid_identifier(&dispatch_message_id)
        .then_some(())
        .ok_or(SchedulerJobEnvelopeBuildErrorV1::InvalidDispatchId)?;
    fire_key
        .iter()
        .any(|byte| *byte != 0)
        .then_some(())
        .ok_or(SchedulerJobEnvelopeBuildErrorV1::InvalidFireKey)?;
    let payload = build_scheduled_job_command_v1(schedule, lease, scheduled_for, trigger)
        .map_err(|_| SchedulerJobEnvelopeBuildErrorV1::InvalidCommand)?
        .encode_to_vec();
    let envelope = envelope(
        schedule,
        lease,
        scheduled_for,
        dispatch_message_id,
        fire_key,
        source,
        payload,
    )?;
    validate_envelope_v1(&envelope)
        .map_err(|_| SchedulerJobEnvelopeBuildErrorV1::InvalidEnvelope)?;
    Ok(envelope)
}

fn envelope(
    schedule: &ScheduleSpecV1,
    lease: &ScheduleRunLeaseV1,
    scheduled_for: UtcMillisV1,
    dispatch_message_id: [u8; 16],
    fire_key: [u8; 32],
    source: &SchedulerDispatchIdentityV1,
    payload: Vec<u8>,
) -> Result<DurableEnvelopeV1, SchedulerJobEnvelopeBuildErrorV1> {
    let binding = schedule.binding();
    let job = binding.job_kind();
    Ok(DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: dispatch_message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: job.owner().to_owned(),
            name: job.name().to_owned(),
            major: u32::from(job.major()),
            revision: 1,
            schema_sha256: binding.schema_sha256().to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: source.runtime_id().to_owned(),
            runtime_instance_id: source.runtime_instance_id().to_vec(),
            runtime_generation: source.runtime_generation(),
        }),
        recorded_at: Some(timestamp(scheduled_for)?),
        partition_key: schedule.scope().value().as_bytes().to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: lease.run_id().bytes().to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: source.runtime_id().as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: source.runtime_id().as_bytes().to_vec(),
            epoch: source.runtime_generation(),
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: lease.run_id().bytes().to_vec(),
            target_capability: "job_execute".to_owned(),
            idempotency_key: fire_key.to_vec(),
            deadline: Some(timestamp(lease.expires_at())?),
            logical_attempt: u32::try_from(lease.epoch())
                .map_err(|_| SchedulerJobEnvelopeBuildErrorV1::InvalidCommand)?,
        })),
        payload,
    })
}

fn timestamp(value: UtcMillisV1) -> Result<Timestamp, SchedulerJobEnvelopeBuildErrorV1> {
    let milliseconds = value.value();
    Ok(Timestamp {
        seconds: milliseconds.div_euclid(1_000),
        nanos: i32::try_from(milliseconds.rem_euclid(1_000) * 1_000_000)
            .map_err(|_| SchedulerJobEnvelopeBuildErrorV1::InvalidTimestamp)?,
    })
}

fn valid_identifier(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
