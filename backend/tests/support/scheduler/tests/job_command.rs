//! Scheduler wire-command contract tests without owner implementation code.

use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, JobRunIdV1, OpaqueOwnerJobScopeV1,
    OpaqueScheduleScopeV1, OwnerJobCommandValidationErrorV1, OwnerJobLeaseV1,
    SCHEDULER_JOB_DESCRIPTOR_SET_V1, ScheduleIdV1, ScheduleRevisionV1, ScheduleRunLeaseV1,
    ScheduleSpecV1, SchedulerCommandValidationErrorV1, build_owner_job_command_v1,
    build_scheduled_job_command_v1, v1, validate_owner_job_command_v1,
    validate_scheduled_job_command_v1,
};
use prost::Message;

#[test]
fn scheduler_command_accepts_a_complete_fenced_time_trigger() {
    assert_eq!(validate_scheduled_job_command_v1(&command()), Ok(()));
}

#[test]
fn scheduler_command_rejects_a_lease_for_another_run() {
    let mut value = command();
    value.lease.as_mut().expect("lease").run_id = vec![9; 16];
    assert_eq!(
        validate_scheduled_job_command_v1(&value),
        Err(SchedulerCommandValidationErrorV1::InvalidLease)
    );
}

#[test]
fn scheduler_command_rejects_unknown_trigger_kind() {
    let mut value = command();
    value.trigger_kind = 99;
    assert_eq!(
        validate_scheduled_job_command_v1(&value),
        Err(SchedulerCommandValidationErrorV1::InvalidTrigger)
    );
}

#[test]
fn scheduler_command_builder_preserves_schedule_and_lease_fences() {
    let schedule = schedule();
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([3; 16]).expect("run"),
        schedule.schedule_id(),
        schedule.revision(),
        2,
        UtcMillisV1::new(2_000),
    )
    .expect("lease");
    let command = build_scheduled_job_command_v1(
        &schedule,
        &lease,
        UtcMillisV1::new(1_000),
        v1::JobTriggerKindV1::Time,
    )
    .expect("command");
    assert_eq!(command.job_run_id, vec![3; 16]);
    assert_eq!(command.lease.expect("lease").epoch, 2);
}

#[test]
fn owner_job_command_accepts_only_the_upgrade_reconciliation_trigger() {
    let command = owner_command();
    assert_eq!(validate_owner_job_command_v1(&command), Ok(()));

    let mut unspecified = command;
    unspecified.trigger_kind = v1::OwnerJobTriggerKindV1::Unspecified as i32;
    assert_eq!(
        validate_owner_job_command_v1(&unspecified),
        Err(OwnerJobCommandValidationErrorV1::InvalidTrigger)
    );
}

#[test]
fn owner_job_command_rejects_zero_run_identity_and_stale_lease() {
    let mut zero_run = owner_command();
    zero_run.job_run_id = vec![0; 16];
    zero_run.lease.as_mut().expect("lease").run_id = vec![0; 16];
    assert_eq!(
        validate_owner_job_command_v1(&zero_run),
        Err(OwnerJobCommandValidationErrorV1::InvalidRun)
    );

    let mut stale_lease = owner_command();
    stale_lease
        .lease
        .as_mut()
        .expect("lease")
        .expires_at_unix_millis = stale_lease.accepted_at_unix_millis;
    assert_eq!(
        validate_owner_job_command_v1(&stale_lease),
        Err(OwnerJobCommandValidationErrorV1::InvalidLease)
    );
}

#[test]
fn owner_job_builder_has_no_schedule_identity() {
    let kind =
        JobKindV1::new("telegram".into(), "calls_realtime_backfill".into(), 1).expect("kind");
    let scope = OpaqueOwnerJobScopeV1::new("owner".into()).expect("scope");
    let lease = OwnerJobLeaseV1::new(
        JobRunIdV1::new([7; 16]).expect("run"),
        3,
        UtcMillisV1::new(2_000),
    )
    .expect("lease");
    let command = build_owner_job_command_v1(
        &kind,
        &scope,
        v1::OwnerJobTriggerKindV1::UpgradeReconciliation,
        UtcMillisV1::new(1_000),
        lease,
    )
    .expect("command");
    assert_eq!(command.job_run_id, vec![7; 16]);
    assert_eq!(command.scope_id, "owner");
    assert_eq!(command.lease.expect("lease").epoch, 3);
}

#[test]
fn owner_and_scheduled_wire_payloads_do_not_interchange() {
    let owner_bytes = owner_command().encode_to_vec();
    let owner_as_scheduled = v1::ScheduledJobCommandV1::decode(owner_bytes.as_slice());
    assert!(match owner_as_scheduled {
        Ok(value) => validate_scheduled_job_command_v1(&value).is_err(),
        Err(_) => true,
    });

    let scheduled_bytes = command().encode_to_vec();
    let scheduled_as_owner = v1::OwnerJobCommandV1::decode(scheduled_bytes.as_slice());
    assert!(match scheduled_as_owner {
        Ok(value) => validate_owner_job_command_v1(&value).is_err(),
        Err(_) => true,
    });
}

#[test]
fn scheduler_job_descriptor_set_is_present_for_exact_contract_binding() {
    assert!(SCHEDULER_JOB_DESCRIPTOR_SET_V1.len() > 32);
}

fn command() -> v1::ScheduledJobCommandV1 {
    v1::ScheduledJobCommandV1 {
        job_run_id: vec![1; 16],
        job_kind: Some(v1::JobKindV1 {
            owner: "mail".into(),
            name: "fetch".into(),
            major: 1,
        }),
        schedule_id: vec![2; 16],
        schedule_revision: 1,
        scope_id: "scope:opaque_42".into(),
        trigger_kind: v1::JobTriggerKindV1::Time as i32,
        scheduled_for_unix_millis: 1_000,
        lease: Some(v1::JobLeaseV1 {
            run_id: vec![1; 16],
            epoch: 1,
            expires_at_unix_millis: 2_000,
        }),
    }
}

fn owner_command() -> v1::OwnerJobCommandV1 {
    v1::OwnerJobCommandV1 {
        job_run_id: vec![7; 16],
        job_kind: Some(v1::JobKindV1 {
            owner: "telegram".into(),
            name: "calls_realtime_backfill".into(),
            major: 1,
        }),
        scope_id: "owner".into(),
        trigger_kind: v1::OwnerJobTriggerKindV1::UpgradeReconciliation as i32,
        accepted_at_unix_millis: 1_000,
        lease: Some(v1::JobLeaseV1 {
            run_id: vec![7; 16],
            epoch: 1,
            expires_at_unix_millis: 2_000,
        }),
    }
}

fn schedule() -> ScheduleSpecV1 {
    let kind = JobKindV1::new("mail".into(), "fetch".into(), 1).expect("kind");
    let binding =
        JobContractBindingV1::new(kind, "mail.fetch".into(), 1, [7; 32]).expect("binding");
    let policy = makosh_scheduler_protocol::SchedulePolicyV1::new(
        makosh_scheduler_protocol::ScheduleTriggerV1::FixedInterval {
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
