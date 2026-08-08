use makosh_clock_protocol::{TimeZoneContextV1, UtcMillisV1};
use makosh_clock_runtime::DeterministicClockV1;
use makosh_scheduler::{
    DueOverlapDecisionV1, ScheduleCatalogErrorV1, ScheduleCatalogV1, ScheduleContinuationV1,
    ScheduleLeaseStateV1, SchedulePlanErrorV1, ScheduleReconcileOutcomeV1, decide_due_overlap,
    plan_due,
};
use makosh_scheduler_persistence::scheduler_storage_bundle_v1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, JobRunIdV1, MisfirePolicyV1,
    OpaqueScheduleScopeV1, OverlapPolicyV1, RetryPolicyV1, ScheduleErrorV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleSpecV1, ScheduleTriggerV1,
};
use makosh_storage_migrations::admit_storage_bundle;
use makosh_storage_protocol::validation::validate_storage_bundle;

#[test]
fn job_contract_binding_rejects_unversioned_names_and_empty_schema_identity() {
    assert!(JobKindV1::new("Notes".into(), "sync".into(), 1).is_err());
    let kind = JobKindV1::new("notes".into(), "sync".into(), 1).expect("job kind");
    assert!(JobContractBindingV1::new(kind.clone(), "Notes.Sync".into(), 1, [1; 32]).is_err());
    assert!(JobContractBindingV1::new(kind.clone(), "notes.sync".into(), 0, [1; 32]).is_err());
    assert!(JobContractBindingV1::new(kind, "notes.sync".into(), 1, [0; 32]).is_err());
}

#[test]
fn fixed_schedule_requires_bounded_overlap_misfire_retry_and_deadline() {
    let retry = RetryPolicyV1::new(3, 1_000).expect("bounded retry");
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        OverlapPolicyV1::AllowBounded { max_parallelism: 2 },
        MisfirePolicyV1::CatchUpBounded { max_runs: 2 },
        retry,
        30_000,
        1_000,
    )
    .expect("bounded schedule policy");
    assert_eq!(policy.retry().max_attempts(), 3);
    assert_eq!(policy.max_parallelism(), 2);
    assert_eq!(policy.deadline_millis(), 30_000);
    assert_eq!(policy.jitter_millis(), 1_000);
    assert_eq!(
        SchedulePolicyV1::new(
            ScheduleTriggerV1::FixedDelay { delay_millis: 1 },
            OverlapPolicyV1::Forbid,
            MisfirePolicyV1::Skip,
            retry,
            1_000,
            0,
        ),
        Err(ScheduleErrorV1::InvalidTrigger)
    );
    assert_eq!(
        SchedulePolicyV1::new(
            ScheduleTriggerV1::FixedInterval {
                interval_millis: 60_000,
            },
            OverlapPolicyV1::Queue {
                max_pending_runs: 0,
            },
            MisfirePolicyV1::Skip,
            retry,
            1_000,
            0,
        ),
        Err(ScheduleErrorV1::InvalidOverlapPolicy)
    );
}

#[test]
fn retry_backoff_is_bounded_and_never_creates_a_fourth_attempt() {
    let retry = RetryPolicyV1::new(3, 1_000).expect("retry");
    assert_eq!(retry.delay_after_failure(1), Some(1_000));
    assert_eq!(retry.delay_after_failure(2), Some(2_000));
    assert_eq!(retry.delay_after_failure(3), None);
    assert_eq!(retry.delay_after_failure(0), None);
}

#[test]
fn canonical_policy_preserves_bounded_queue_configuration() {
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        OverlapPolicyV1::Queue {
            max_pending_runs: 3,
        },
        MisfirePolicyV1::FireOnce,
        RetryPolicyV1::new(2, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("queue policy");
    assert_eq!(
        SchedulePolicyV1::from_canonical_bytes(&policy.canonical_bytes()),
        Ok(policy)
    );
}

#[test]
fn cron_schedule_requires_explicit_timezone_and_opaque_scope() {
    let timezone = TimeZoneContextV1::new("Europe/Madrid".into(), 7_200).expect("timezone");
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::Cron {
            expression: "0 9 * * 1-5".into(),
            timezone,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::FireOnce,
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("cron schedule policy");
    let kind = JobKindV1::new("notes".into(), "sync".into(), 1).expect("job kind");
    let binding =
        JobContractBindingV1::new(kind, "notes.sync".into(), 1, [7; 32]).expect("binding");
    let schedule = ScheduleSpecV1::new(
        ScheduleIdV1::new([1; 16]).expect("schedule id"),
        ScheduleRevisionV1::new(1).expect("revision"),
        binding,
        OpaqueScheduleScopeV1::new("scope:9f84".into()).expect("opaque scope"),
        ConcurrencyKeyV1::new("mailbox:opaque_9f84".into()).expect("concurrency key"),
        true,
        policy,
    );
    assert!(schedule.enabled());
    assert_eq!(
        ConcurrencyKeyV1::new("mailbox:user@example.test".into()),
        Err(ScheduleErrorV1::InvalidConcurrencyKey)
    );
}

#[test]
fn deterministic_clock_expires_stale_schedule_run_lease() {
    let mut clock = DeterministicClockV1::new(1_000);
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([2; 16]).expect("run id"),
        ScheduleIdV1::new([1; 16]).expect("schedule id"),
        ScheduleRevisionV1::new(1).expect("revision"),
        4,
        UtcMillisV1::new(1_100),
    )
    .expect("run lease");
    assert!(lease.accepts_completion(4, clock.read().wall_utc()));
    clock.advance(100);
    assert!(!lease.accepts_completion(4, clock.read().wall_utc()));
    assert!(!lease.accepts_completion(5, UtcMillisV1::new(1_050)));
}

#[test]
fn reconciliation_is_revisioned_and_preserves_an_in_flight_lease() {
    let first = schedule(1, true);
    let mut catalog = ScheduleCatalogV1::default();
    assert_eq!(
        catalog
            .reconcile(first.clone(), UtcMillisV1::new(1_000))
            .expect("insert"),
        ScheduleReconcileOutcomeV1::Inserted
    );
    assert_eq!(
        catalog
            .reconcile(first.clone(), UtcMillisV1::new(2_000))
            .expect("repeat"),
        ScheduleReconcileOutcomeV1::Unchanged
    );
    let acquired_lease = lease(1, 1, 3_000);
    catalog
        .claim_due(acquired_lease, UtcMillisV1::new(1_000))
        .expect("claim");
    assert_eq!(
        catalog
            .reconcile(schedule(2, true), UtcMillisV1::new(2_000))
            .expect("update"),
        ScheduleReconcileOutcomeV1::Updated
    );
    assert!(matches!(
        catalog
            .entry(ScheduleIdV1::new([1; 16]).expect("schedule id"))
            .expect("entry")
            .lease_state(UtcMillisV1::new(1_050)),
        ScheduleLeaseStateV1::Active(_)
    ));
    assert_eq!(
        catalog.claim_due(lease(2, 2, 3_000), UtcMillisV1::new(2_000)),
        Err(ScheduleCatalogErrorV1::ClaimDenied)
    );
}

#[test]
fn scheduler_bundle_has_owner_scoped_schedule_and_run_schema() {
    let bundle = scheduler_storage_bundle_v1();
    validate_storage_bundle(&bundle).expect("structural bundle validation");
    admit_storage_bundle(&bundle).expect("PostgreSQL AST admission");
    let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8).expect("SQL");
    assert!(sql.contains("makosh_platform.scheduler_schedules"));
    assert!(sql.contains("makosh_platform.scheduler_runs"));
    assert!(sql.contains("makosh_platform.scheduler_concurrency"));
    assert!(bundle.steps.iter().any(|step| {
        std::str::from_utf8(&step.forward_sql_utf8)
            .is_ok_and(|sql| sql.contains("makosh_platform.scheduler_pending_fires"))
    }));
    assert!(sql.contains("concurrency_key TEXT NOT NULL"));
    assert!(sql.contains("fire_key BYTEA NOT NULL UNIQUE"));
}

#[test]
fn same_scope_is_serialized_while_independent_mailboxes_run_concurrently() {
    let mut catalog = ScheduleCatalogV1::default();
    reconcile(
        &mut catalog,
        schedule_with(1, 1, "mailbox:opaque_a", OverlapPolicyV1::Forbid),
    );
    reconcile(
        &mut catalog,
        schedule_with(2, 1, "mailbox:opaque_a", OverlapPolicyV1::Forbid),
    );
    reconcile(
        &mut catalog,
        schedule_with(3, 1, "mailbox:opaque_b", OverlapPolicyV1::Forbid),
    );

    catalog
        .claim_due(lease_for(11, 1, 1, 3_000), UtcMillisV1::new(1_000))
        .expect("first claim");
    assert_eq!(
        catalog.claim_due(lease_for(12, 2, 1, 3_000), UtcMillisV1::new(1_000)),
        Err(ScheduleCatalogErrorV1::ClaimDenied)
    );
    catalog
        .claim_due(lease_for(13, 3, 1, 3_000), UtcMillisV1::new(1_000))
        .expect("independent claim");
}

#[test]
fn bounded_policy_limits_parallel_runs_for_one_coordination_key() {
    let mut catalog = ScheduleCatalogV1::default();
    let policy = OverlapPolicyV1::AllowBounded { max_parallelism: 2 };
    for id in 1..=3 {
        reconcile(
            &mut catalog,
            schedule_with(id, 1, "mailbox:opaque_shared", policy),
        );
    }
    catalog
        .claim_due(lease_for(21, 1, 1, 3_000), UtcMillisV1::new(1_000))
        .expect("first claim");
    catalog
        .claim_due(lease_for(22, 2, 1, 3_000), UtcMillisV1::new(1_000))
        .expect("second claim");
    assert_eq!(
        catalog.claim_due(lease_for(23, 3, 1, 3_000), UtcMillisV1::new(1_000)),
        Err(ScheduleCatalogErrorV1::ClaimDenied)
    );
}

#[test]
fn overlap_policies_keep_slow_runs_bounded_without_blocking_other_mailboxes() {
    assert_eq!(
        decide_due_overlap(OverlapPolicyV1::Forbid, 1, 0),
        DueOverlapDecisionV1::Drop
    );
    assert_eq!(
        decide_due_overlap(
            OverlapPolicyV1::Queue {
                max_pending_runs: 2,
            },
            1,
            0,
        ),
        DueOverlapDecisionV1::Enqueue
    );
    assert_eq!(
        decide_due_overlap(
            OverlapPolicyV1::Queue {
                max_pending_runs: 2,
            },
            1,
            2,
        ),
        DueOverlapDecisionV1::Drop
    );
    assert_eq!(
        decide_due_overlap(OverlapPolicyV1::CoalesceLatest, 1, 1),
        DueOverlapDecisionV1::ReplacePending
    );
    assert_eq!(
        decide_due_overlap(OverlapPolicyV1::AllowBounded { max_parallelism: 2 }, 1, 0),
        DueOverlapDecisionV1::Start
    );
}

#[test]
fn interval_misfire_never_replays_an_unbounded_backlog() {
    let due = UtcMillisV1::new(0);
    let now = UtcMillisV1::new(25_000);
    let skipped = plan_due(&policy_with_misfire(MisfirePolicyV1::Skip), due, now).expect("plan");
    assert!(skipped.occurrences().is_empty());
    assert_eq!(
        skipped.continuation(),
        ScheduleContinuationV1::At(UtcMillisV1::new(30_000))
    );
    let once = plan_due(&policy_with_misfire(MisfirePolicyV1::FireOnce), due, now).expect("plan");
    assert_eq!(once.occurrences(), [UtcMillisV1::new(20_000)]);
    let catch_up = plan_due(
        &policy_with_misfire(MisfirePolicyV1::CatchUpBounded { max_runs: 2 }),
        due,
        now,
    )
    .expect("plan");
    assert_eq!(
        catch_up.occurrences(),
        [UtcMillisV1::new(10_000), UtcMillisV1::new(20_000)]
    );
}

#[test]
fn fixed_delay_waits_for_terminal_completion_before_rearming() {
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedDelay {
            delay_millis: 60_000,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::FireOnce,
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("fixed delay policy");
    let plan = plan_due(&policy, UtcMillisV1::new(1_000), UtcMillisV1::new(100_000)).expect("plan");
    assert_eq!(plan.occurrences(), [UtcMillisV1::new(1_000)]);
    assert_eq!(
        plan.continuation(),
        ScheduleContinuationV1::AfterTerminalDelay(60_000)
    );
}

#[test]
fn cron_is_explicitly_rejected_until_dst_execution_is_implemented() {
    let timezone = TimeZoneContextV1::new("Europe/Madrid".into(), 7_200).expect("timezone");
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::Cron {
            expression: "0 9 * * 1-5".into(),
            timezone,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::Skip,
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("cron policy");
    assert_eq!(
        plan_due(&policy, UtcMillisV1::new(0), UtcMillisV1::new(1)),
        Err(SchedulePlanErrorV1::CronNotImplemented)
    );
}

fn schedule(revision: u64, enabled: bool) -> ScheduleSpecV1 {
    let mut spec = schedule_with(1, revision, "scheduler:technical", OverlapPolicyV1::Forbid);
    if !enabled {
        spec = ScheduleSpecV1::new(
            spec.schedule_id(),
            spec.revision(),
            spec.binding().clone(),
            spec.scope().clone(),
            spec.concurrency_key().clone(),
            false,
            spec.policy().clone(),
        );
    }
    spec
}

fn schedule_with(
    id: u8,
    revision: u64,
    concurrency_key: &str,
    overlap: OverlapPolicyV1,
) -> ScheduleSpecV1 {
    let kind = JobKindV1::new("platform".into(), "maintenance".into(), 1).expect("kind");
    let binding = JobContractBindingV1::new(kind, "platform.maintenance".into(), 1, [7; 32])
        .expect("binding");
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        overlap,
        MisfirePolicyV1::Skip,
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy");
    ScheduleSpecV1::new(
        ScheduleIdV1::new([id; 16]).expect("schedule id"),
        ScheduleRevisionV1::new(revision).expect("revision"),
        binding,
        OpaqueScheduleScopeV1::new("scope:technical".into()).expect("scope"),
        ConcurrencyKeyV1::new(concurrency_key.into()).expect("concurrency key"),
        true,
        policy,
    )
}

fn policy_with_misfire(misfire: MisfirePolicyV1) -> SchedulePolicyV1 {
    SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 10_000,
        },
        OverlapPolicyV1::Forbid,
        misfire,
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("interval policy")
}

fn lease(revision: u64, epoch: u64, expires_at: i64) -> ScheduleRunLeaseV1 {
    ScheduleRunLeaseV1::new(
        JobRunIdV1::new([revision as u8; 16]).expect("run id"),
        ScheduleIdV1::new([1; 16]).expect("schedule id"),
        ScheduleRevisionV1::new(revision).expect("revision"),
        epoch,
        UtcMillisV1::new(expires_at),
    )
    .expect("lease")
}

fn lease_for(run: u8, schedule: u8, revision: u64, expires_at: i64) -> ScheduleRunLeaseV1 {
    ScheduleRunLeaseV1::new(
        JobRunIdV1::new([run; 16]).expect("run id"),
        ScheduleIdV1::new([schedule; 16]).expect("schedule id"),
        ScheduleRevisionV1::new(revision).expect("revision"),
        1,
        UtcMillisV1::new(expires_at),
    )
    .expect("lease")
}

fn reconcile(catalog: &mut ScheduleCatalogV1, spec: ScheduleSpecV1) {
    catalog
        .reconcile(spec, UtcMillisV1::new(1_000))
        .expect("schedule insert");
}
