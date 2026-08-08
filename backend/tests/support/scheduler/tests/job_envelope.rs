//! Scheduler dispatch envelope tests without owner runtime code.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::v1::durable_envelope_v1::Semantics;
use makosh_scheduler::{
    SchedulerDispatchIdentityV1, SchedulerJobEnvelopeBuildErrorV1, build_scheduled_job_envelope_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, JobRunIdV1, OpaqueScheduleScopeV1,
    ScheduleIdV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleSpecV1, ScheduleTriggerV1, v1,
};
use prost::Message;

#[test]
fn scheduler_envelope_binds_message_command_run_and_runtime_fences() {
    let schedule = schedule();
    let lease = lease(&schedule);
    let envelope = build_scheduled_job_envelope_v1(
        &schedule,
        &lease,
        UtcMillisV1::new(1_000),
        v1::JobTriggerKindV1::Time,
        [4; 16],
        [5; 32],
        &identity(),
    )
    .expect("envelope");
    let command = match envelope.semantics.as_ref() {
        Some(Semantics::Command(value)) => value,
        _ => panic!("command metadata"),
    };
    assert_eq!(envelope.message_id, vec![4; 16]);
    assert_eq!(envelope.correlation_id, vec![3; 16]);
    assert_eq!(command.command_id, vec![3; 16]);
    assert_eq!(command.idempotency_key, vec![5; 32]);
    assert_eq!(command.logical_attempt, 2);
    let payload = v1::ScheduledJobCommandV1::decode(envelope.payload.as_slice()).expect("payload");
    assert_eq!(payload.job_run_id, vec![3; 16]);
    assert_eq!(payload.lease.expect("lease").epoch, 2);
}

#[test]
fn scheduler_envelope_rejects_a_zero_dispatch_identity() {
    assert_eq!(
        build_scheduled_job_envelope_v1(
            &schedule(),
            &lease(&schedule()),
            UtcMillisV1::new(1_000),
            v1::JobTriggerKindV1::Time,
            [0; 16],
            [5; 32],
            &identity(),
        ),
        Err(SchedulerJobEnvelopeBuildErrorV1::InvalidDispatchId)
    );
}

fn identity() -> SchedulerDispatchIdentityV1 {
    SchedulerDispatchIdentityV1::new("scheduler_runtime".into(), [8; 16], 4).expect("identity")
}

fn lease(schedule: &ScheduleSpecV1) -> ScheduleRunLeaseV1 {
    ScheduleRunLeaseV1::new(
        JobRunIdV1::new([3; 16]).expect("run"),
        schedule.schedule_id(),
        schedule.revision(),
        2,
        UtcMillisV1::new(2_000),
    )
    .expect("lease")
}

fn schedule() -> ScheduleSpecV1 {
    let kind = JobKindV1::new("mail".into(), "fetch".into(), 1).expect("kind");
    let binding =
        JobContractBindingV1::new(kind, "mail.fetch".into(), 1, [7; 32]).expect("binding");
    let policy = makosh_scheduler_protocol::SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        makosh_scheduler_protocol::OverlapPolicyV1::Forbid,
        makosh_scheduler_protocol::MisfirePolicyV1::Skip,
        makosh_scheduler_protocol::RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy");
    ScheduleSpecV1::new(
        ScheduleIdV1::new([2; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        binding,
        OpaqueScheduleScopeV1::new("scope:opaque_42".into()).expect("scope"),
        ConcurrencyKeyV1::new("mailbox:opaque_42".into()).expect("key"),
        true,
        policy,
    )
}
