//! Disposable PostgreSQL proof for Scheduler concurrency slot fencing.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::{OutboxPublishReceiptV1, OutboxRecordV1, OwnerOutboxStorePortV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1,
        ResultMetadataV1, ResultOutcomeV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_persistence::{
    FixedDelayCompletionOutcomeV1, SchedulerPendingFireOutcomeV1, SchedulerPendingFireV1,
    SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1,
    SchedulerScheduleControlApplyErrorV1, SchedulerScheduleControlApplyOutcomeV1,
    SchedulerScheduleControlAuthorityV1, SchedulerScheduleControlDecisionV1,
    SchedulerScheduleControlMutationV1, SchedulerScheduleControlRejectionV1,
    SchedulerScheduleControlRequestV1, SchedulerScheduleControlResultOutboxV1,
    SchedulerScheduleStoreErrorV1, SchedulerScheduleUpsertOutcomeV1, SchedulerScheduleUpsertV1,
    scheduler_storage_bundle_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, JobKindV1, JobRunIdV1, MisfirePolicyV1,
    OpaqueScheduleScopeV1, OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1, SchedulePolicyV1,
    ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleSpecV1, ScheduleTriggerV1,
};
use prost::Message;
use prost_types::Timestamp;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const POSTGRES_URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";
const CLAIMED_AT: i64 = 1_000;

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn shared_slots_serialize_one_resource_and_allow_independent_resources() {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect disposable Scheduler PostgreSQL");
    install_schema(&pool).await;
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    assert_schedule_control_is_atomic_and_cancel_is_race_safe(&pool, &store).await;
    assert_revisioned_schedule_configuration(&pool, &store).await;
    assert_pending_queue_and_coalescing(&pool, &store).await;
    assert_fixed_delay_rearms_at_terminal_completion(&pool, &store).await;
    let forbid_policy = policy(OverlapPolicyV1::Forbid);
    let same = key("mailbox:opaque_shared");
    let other = key("mailbox:opaque_other");
    store
        .ensure_concurrency_slot(&same, &forbid_policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("configure shared slot");
    store
        .ensure_concurrency_slot(&other, &forbid_policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("configure independent slot");
    install_schedule(&pool, 1, &same, 1).await;
    install_schedule(&pool, 2, &same, 1).await;
    install_schedule(&pool, 3, &other, 1).await;

    assert_shared_key_race(&pool, &store, &forbid_policy, &same, other).await;
    assert_expired_lease_releases_slot(&pool, &store, &forbid_policy, same).await;
    assert_bounded_parallelism(&pool, &store).await;
}

async fn assert_schedule_control_is_atomic_and_cancel_is_race_safe(
    pool: &PgPool,
    store: &SchedulerPostgresStoreV1,
) {
    let invalid_result_request = control_request(
        58,
        59,
        SchedulerScheduleControlMutationV1::Ensure(Box::new(config_change(
            59,
            1,
            1,
            key("control:opaque_59"),
            policy(OverlapPolicyV1::Forbid),
            5_000,
            1_000,
        ))),
    );
    assert_eq!(
        store
            .apply_schedule_control(&invalid_result_request, |_| {
                Ok(envelope_record(59, true, 0, b"not-a-result"))
            })
            .await,
        Err(SchedulerScheduleControlApplyErrorV1::InvalidResult)
    );
    let rolled_back: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1",
    )
    .bind(vec![59_u8; 16])
    .fetch_one(pool)
    .await
    .expect("verify invalid result rollback");
    assert_eq!(rolled_back, 0);

    let schedule_key = key("control:opaque_60");
    let change = config_change(
        60,
        1,
        1,
        schedule_key,
        policy(OverlapPolicyV1::Forbid),
        5_000,
        1_000,
    );
    let ensure = control_request(
        60,
        60,
        SchedulerScheduleControlMutationV1::Ensure(Box::new(change)),
    );
    let applied = store
        .apply_schedule_control(&ensure, |decision| {
            assert_eq!(decision, SchedulerScheduleControlDecisionV1::Ensured);
            Ok(envelope_record(61, false, 60, b"ensured"))
        })
        .await
        .expect("atomic ensure and result");
    assert!(matches!(
        applied,
        SchedulerScheduleControlApplyOutcomeV1::Applied {
            decision: SchedulerScheduleControlDecisionV1::Ensured,
            ..
        }
    ));
    let mut result_outbox = SchedulerScheduleControlResultOutboxV1::new(store);
    let result_entry = result_outbox
        .next_pending()
        .await
        .expect("read exact pending result")
        .expect("pending result");
    assert_eq!(result_entry.record().message_id(), &[61; 16]);
    result_outbox
        .mark_published(
            &result_entry,
            &OutboxPublishReceiptV1::new("SCHEDULER_RESULTS", 1, false).expect("publish receipt"),
        )
        .await
        .expect("mark exact result published");
    assert!(
        result_outbox
            .next_pending()
            .await
            .expect("read empty result outbox")
            .is_none()
    );
    let duplicate = store
        .apply_schedule_control(&ensure, |_| panic!("duplicate reuses exact result"))
        .await
        .expect("duplicate command");
    assert!(matches!(
        duplicate,
        SchedulerScheduleControlApplyOutcomeV1::Duplicate {
            decision: SchedulerScheduleControlDecisionV1::Ensured,
            ..
        }
    ));

    let conflicting = SchedulerScheduleControlRequestV1::new(
        envelope_record(60, true, 0, b"different"),
        [60; 16],
        ensure.authority().clone(),
        ensure.mutation().clone(),
        UtcMillisV1::new(1_001),
    )
    .expect("same-ID conflicting command");
    assert_eq!(
        store
            .apply_schedule_control(&conflicting, |_| {
                Ok(envelope_record(62, false, 60, b"must-not-persist"))
            })
            .await,
        Err(SchedulerScheduleControlApplyErrorV1::HashConflict)
    );

    let foreign_authority = SchedulerScheduleControlAuthorityV1::new(
        "foreign.runtime.v1".to_owned(),
        "platform".to_owned(),
        "platform".to_owned(),
        "maintenance".to_owned(),
        1,
    )
    .expect("foreign schedule authority");
    let foreign_cancel = SchedulerScheduleControlRequestV1::new(
        envelope_record(68, true, 0, b"foreign-cancel"),
        [68; 16],
        foreign_authority,
        SchedulerScheduleControlMutationV1::Cancel {
            schedule_id: ScheduleIdV1::new([60; 16]).expect("schedule ID"),
            expected_revision: ScheduleRevisionV1::new(1).expect("revision"),
            cancelled_at: UtcMillisV1::new(1_050),
        },
        UtcMillisV1::new(1_050),
    )
    .expect("foreign cancellation request");
    assert!(matches!(
        store
            .apply_schedule_control(&foreign_cancel, |decision| {
                assert_eq!(
                    decision,
                    SchedulerScheduleControlDecisionV1::Rejected(
                        SchedulerScheduleControlRejectionV1::ForeignAuthority
                    )
                );
                Ok(envelope_record(69, false, 68, b"foreign-authority"))
            })
            .await,
        Ok(SchedulerScheduleControlApplyOutcomeV1::Applied {
            decision: SchedulerScheduleControlDecisionV1::Rejected(
                SchedulerScheduleControlRejectionV1::ForeignAuthority
            ),
            ..
        })
    ));

    let cancel = control_request(
        62,
        60,
        SchedulerScheduleControlMutationV1::Cancel {
            schedule_id: ScheduleIdV1::new([60; 16]).expect("schedule ID"),
            expected_revision: ScheduleRevisionV1::new(1).expect("revision"),
            cancelled_at: UtcMillisV1::new(1_100),
        },
    );
    assert!(matches!(
        store
            .apply_schedule_control(&cancel, |decision| {
                assert_eq!(decision, SchedulerScheduleControlDecisionV1::Cancelled);
                Ok(envelope_record(63, false, 62, b"cancelled"))
            })
            .await,
        Ok(SchedulerScheduleControlApplyOutcomeV1::Applied {
            decision: SchedulerScheduleControlDecisionV1::Cancelled,
            ..
        })
    ));

    let race_key = key("control:opaque_61");
    let race_policy = policy(OverlapPolicyV1::Forbid);
    let race_change = config_change(
        61,
        1,
        1,
        race_key.clone(),
        race_policy.clone(),
        CLAIMED_AT,
        CLAIMED_AT,
    );
    store
        .apply_schedule_control(
            &control_request(
                64,
                61,
                SchedulerScheduleControlMutationV1::Ensure(Box::new(race_change)),
            ),
            |_| Ok(envelope_record(65, false, 64, b"ensured")),
        )
        .await
        .expect("ensure race schedule");
    store
        .claim_due(&claim(81, 61, race_key, &race_policy))
        .await
        .expect("durably accept due run");
    let too_late = control_request(
        66,
        61,
        SchedulerScheduleControlMutationV1::Cancel {
            schedule_id: ScheduleIdV1::new([61; 16]).expect("schedule ID"),
            expected_revision: ScheduleRevisionV1::new(1).expect("revision"),
            cancelled_at: UtcMillisV1::new(CLAIMED_AT + 1),
        },
    );
    assert!(matches!(
        store
            .apply_schedule_control(&too_late, |decision| {
                assert_eq!(decision, SchedulerScheduleControlDecisionV1::TooLate);
                Ok(envelope_record(67, false, 66, b"too-late"))
            })
            .await,
        Ok(SchedulerScheduleControlApplyOutcomeV1::Applied {
            decision: SchedulerScheduleControlDecisionV1::TooLate,
            ..
        })
    ));
}

fn control_request(
    command_id: u8,
    schedule_id: u8,
    mutation: SchedulerScheduleControlMutationV1,
) -> SchedulerScheduleControlRequestV1 {
    SchedulerScheduleControlRequestV1::new(
        envelope_record(command_id, true, 0, b"command"),
        [schedule_id; 16],
        control_authority(),
        mutation,
        UtcMillisV1::new(1_000),
    )
    .expect("schedule control request")
}

fn control_authority() -> SchedulerScheduleControlAuthorityV1 {
    SchedulerScheduleControlAuthorityV1::new(
        "communication_delayed_delivery.runtime.v1".to_owned(),
        "platform".to_owned(),
        "platform".to_owned(),
        "maintenance".to_owned(),
        1,
    )
    .expect("schedule control authority")
}

fn envelope_record(
    message_id: u8,
    command: bool,
    causation_message_id: u8,
    payload: &[u8],
) -> OutboxRecordV1 {
    let timestamp = Timestamp {
        seconds: 1,
        nanos: 0,
    };
    let semantics = if command {
        Semantics::Command(CommandMetadataV1 {
            command_id: vec![message_id; 16],
            target_capability: "scheduler_schedule_control".to_owned(),
            idempotency_key: vec![message_id; 16],
            deadline: Some(Timestamp {
                seconds: 60,
                nanos: 0,
            }),
            logical_attempt: 1,
        })
    } else {
        Semantics::Result(ResultMetadataV1 {
            command_id: vec![causation_message_id; 16],
            command_message_id: vec![causation_message_id; 16],
            outcome: ResultOutcomeV1::Succeeded.into(),
            completed_at: Some(timestamp),
            execution_attempt: 1,
        })
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![message_id; 16],
        contract: Some(ContractRefV1 {
            owner: "scheduler".to_owned(),
            name: "schedule_control".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "scheduler".to_owned(),
            runtime_instance_id: vec![9; 16],
            runtime_generation: 1,
        }),
        recorded_at: Some(timestamp),
        partition_key: vec![message_id],
        causation_message_id: if command {
            Vec::new()
        } else {
            vec![causation_message_id; 16]
        },
        correlation_id: vec![8; 16],
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System.into(),
            actor_id: b"scheduler".to_vec(),
        }),
        trace: None,
        source_fence: None,
        semantics: Some(semantics),
        payload: payload.to_vec(),
    };
    OutboxRecordV1::accept(envelope.encode_to_vec()).expect("valid exact envelope record")
}

async fn assert_shared_key_race(
    pool: &PgPool,
    store: &SchedulerPostgresStoreV1,
    policy: &SchedulePolicyV1,
    same: &ConcurrencyKeyV1,
    other: ConcurrencyKeyV1,
) {
    let first = claim(11, 1, same.clone(), policy);
    let second = claim(12, 2, same.clone(), policy);
    let (first_result, second_result) =
        tokio::join!(store.claim_due(&first), store.claim_due(&second));
    assert_one_claim_won(&first_result, &second_result);
    assert_eq!(due_at(pool, 2).await, CLAIMED_AT);
    store
        .claim_due(&claim(13, 3, other, policy))
        .await
        .expect("independent key is not blocked");
    let (winner, blocked) = if first_result.is_ok() {
        (&first, &second)
    } else {
        (&second, &first)
    };
    store
        .finish_claim(winner, UtcMillisV1::new(CLAIMED_AT + 1))
        .await
        .expect("fenced terminal completion releases the slot");
    store
        .claim_due(blocked)
        .await
        .expect("released shared slot");
}

async fn assert_expired_lease_releases_slot(
    pool: &PgPool,
    store: &SchedulerPostgresStoreV1,
    policy: &SchedulePolicyV1,
    same: ConcurrencyKeyV1,
) {
    sqlx::query(
        "UPDATE makosh_platform.scheduler_runs SET lease_expires_at_unix_ms = $1 WHERE concurrency_key = $2 AND state = 'pending_dispatch'",
    )
    .bind(CLAIMED_AT + 2)
    .bind(same.value())
    .execute(pool)
    .await
    .expect("force a stalled worker lease in the disposable contour");
    store
        .claim_due(&claim_at(
            14, 1, same, policy, 61_000, 62_000, 120_000, 72_000,
        ))
        .await
        .expect("expired lease releases the shared slot before the next claim");
}

async fn assert_bounded_parallelism(pool: &PgPool, store: &SchedulerPostgresStoreV1) {
    let bounded = key("mailbox:opaque_bounded");
    let bounded_policy = policy(OverlapPolicyV1::AllowBounded { max_parallelism: 2 });
    store
        .ensure_concurrency_slot(&bounded, &bounded_policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("configure bounded slot");
    for id in 4..=6 {
        install_schedule(pool, id, &bounded, 2).await;
    }
    for (run, schedule) in [(24, 4), (25, 5)] {
        store
            .claim_due(&claim(run, schedule, bounded.clone(), &bounded_policy))
            .await
            .expect("bounded claim");
    }
    assert_eq!(
        store
            .claim_due(&claim(26, 6, bounded, &bounded_policy))
            .await,
        Err(SchedulerRunClaimErrorV1::ConcurrencyExhausted)
    );
}

async fn assert_revisioned_schedule_configuration(pool: &PgPool, store: &SchedulerPostgresStoreV1) {
    let key = key("mailbox:opaque_config");
    let policy = policy(OverlapPolicyV1::Forbid);
    let initial = config_change(7, 1, 1, key.clone(), policy.clone(), 900, 100);
    assert_eq!(
        store.upsert_schedule(&initial).await,
        Ok(SchedulerScheduleUpsertOutcomeV1::Inserted)
    );
    assert_eq!(
        store.upsert_schedule(&initial).await,
        Ok(SchedulerScheduleUpsertOutcomeV1::Unchanged)
    );
    let updated = config_change(7, 2, 2, key.clone(), policy.clone(), 800, 200);
    assert_eq!(
        store.upsert_schedule(&updated).await,
        Ok(SchedulerScheduleUpsertOutcomeV1::Updated)
    );
    let contract_revision: i32 = sqlx::query_scalar(
        "SELECT contract_revision FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1",
    )
    .bind(vec![7_u8; 16])
    .fetch_one(pool)
    .await
    .expect("read persisted contract revision");
    assert_eq!(contract_revision, 2);
    assert_eq!(
        store.upsert_schedule(&initial).await,
        Err(SchedulerScheduleStoreErrorV1::StaleRevision)
    );
    let conflict = config_change(7, 2, 3, key, policy, 801, 200);
    assert_eq!(
        store.upsert_schedule(&conflict).await,
        Err(SchedulerScheduleStoreErrorV1::RevisionConflict)
    );
    let due = store
        .due_schedules(UtcMillisV1::new(800), 16)
        .await
        .expect("load persisted due schedule");
    assert!(due.iter().any(|schedule| {
        schedule.spec().schedule_id().bytes() == [7; 16] && schedule.next_due_at().value() == 800
    }));
}

async fn assert_pending_queue_and_coalescing(pool: &PgPool, store: &SchedulerPostgresStoreV1) {
    let queued = policy(OverlapPolicyV1::Queue {
        max_pending_runs: 2,
    });
    let queue_key = key("mailbox:opaque_pending");
    install_configured_schedule(store, 8, queue_key.clone(), queued.clone()).await;
    let first = pending_fire(claim(31, 8, queue_key.clone(), &queued));
    assert_eq!(
        store.record_pending_fire(&first).await,
        Ok(SchedulerPendingFireOutcomeV1::Queued)
    );
    assert_eq!(
        store.record_pending_fire(&first).await,
        Ok(SchedulerPendingFireOutcomeV1::AlreadyQueued)
    );
    assert_eq!(
        store
            .record_pending_fire(&pending_fire(claim_at(
                32,
                8,
                queue_key.clone(),
                &queued,
                CLAIMED_AT + 1,
                CLAIMED_AT + 1,
                CLAIMED_AT + 60_000,
                CLAIMED_AT + 10_000,
            )))
            .await,
        Ok(SchedulerPendingFireOutcomeV1::Queued)
    );
    assert_eq!(
        store
            .record_pending_fire(&pending_fire(claim_at(
                33,
                8,
                queue_key,
                &queued,
                CLAIMED_AT + 2,
                CLAIMED_AT + 2,
                CLAIMED_AT + 60_000,
                CLAIMED_AT + 10_000,
            )))
            .await,
        Ok(SchedulerPendingFireOutcomeV1::Dropped)
    );
    assert_pending_fire_is_single_use(pool, store, &first).await;

    assert_coalesced_pending_fire(pool, store).await;
}

async fn assert_fixed_delay_rearms_at_terminal_completion(
    pool: &PgPool,
    store: &SchedulerPostgresStoreV1,
) {
    let policy = fixed_delay_policy();
    let key = key("mailbox:opaque_delay");
    install_configured_schedule(store, 10, key.clone(), policy.clone()).await;
    let claim = claim(50, 10, key, &policy);
    store
        .claim_due(&claim)
        .await
        .expect("claim fixed-delay run");
    assert_eq!(due_at(pool, 10).await, CLAIMED_AT);
    assert_eq!(
        store
            .finish_fixed_delay_claim(&claim, UtcMillisV1::new(CLAIMED_AT + 1_000))
            .await,
        Ok(FixedDelayCompletionOutcomeV1::Rearmed)
    );
    assert_eq!(due_at(pool, 10).await, CLAIMED_AT + 61_000);
}

async fn assert_pending_fire_is_single_use(
    pool: &PgPool,
    store: &SchedulerPostgresStoreV1,
    first: &SchedulerPendingFireV1,
) {
    store
        .claim_pending_fire(first.claim())
        .await
        .expect("pending fire becomes one fenced run");
    assert_eq!(pending_count(pool, 8).await, 1);
    store
        .finish_claim(first.claim(), UtcMillisV1::new(CLAIMED_AT + 2))
        .await
        .expect("terminal pending fire releases its slot");
    assert_eq!(
        store.claim_pending_fire(first.claim()).await,
        Err(SchedulerRunClaimErrorV1::PendingMissing)
    );
}

async fn assert_coalesced_pending_fire(pool: &PgPool, store: &SchedulerPostgresStoreV1) {
    let coalesce = policy(OverlapPolicyV1::CoalesceLatest);
    let coalesce_key = key("mailbox:opaque_coalesce");
    install_configured_schedule(store, 9, coalesce_key.clone(), coalesce.clone()).await;
    for (run, scheduled_for) in [(41, CLAIMED_AT), (42, CLAIMED_AT + 1)] {
        let fire = pending_fire(claim_at(
            run,
            9,
            coalesce_key.clone(),
            &coalesce,
            scheduled_for,
            CLAIMED_AT + 1,
            CLAIMED_AT + 60_000,
            CLAIMED_AT + 10_000,
        ));
        assert_eq!(
            store.record_pending_fire(&fire).await,
            Ok(SchedulerPendingFireOutcomeV1::Coalesced)
        );
    }
    assert_eq!(pending_count(pool, 9).await, 1);
    assert_eq!(pending_due_at(pool, 9).await, CLAIMED_AT + 1);
}

async fn install_configured_schedule(
    store: &SchedulerPostgresStoreV1,
    id: u8,
    key: ConcurrencyKeyV1,
    policy: SchedulePolicyV1,
) {
    let change = config_change(id, 1, 1, key, policy, CLAIMED_AT, CLAIMED_AT);
    assert_eq!(
        store.upsert_schedule(&change).await,
        Ok(SchedulerScheduleUpsertOutcomeV1::Inserted)
    );
}

async fn install_schema(pool: &PgPool) {
    let bundle = scheduler_storage_bundle_v1();
    sqlx::raw_sql("CREATE SCHEMA IF NOT EXISTS makosh_platform;")
        .execute(pool)
        .await
        .expect("create Scheduler schema");
    for step in bundle.steps {
        let sql = std::str::from_utf8(&step.forward_sql_utf8).expect("Scheduler migration UTF-8");
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
            .execute(pool)
            .await
            .expect("install Scheduler migration in disposable contour");
    }
}

async fn install_schedule(pool: &PgPool, id: u8, key: &ConcurrencyKeyV1, max: i32) {
    sqlx::query(
        "INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, $4, TRUE, $5, $6, $6)",
    )
    .bind(vec![id; 16])
    .bind(vec![7_u8; 32])
    .bind(key.value())
    .bind(max)
    .bind(vec![1_u8])
    .bind(CLAIMED_AT)
    .execute(pool)
    .await
    .expect("install schedule");
}

fn claim(
    run: u8,
    schedule: u8,
    key: ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
) -> SchedulerRunClaimV1 {
    claim_at(
        run,
        schedule,
        key,
        policy,
        CLAIMED_AT,
        CLAIMED_AT,
        CLAIMED_AT + 60_000,
        CLAIMED_AT + 10_000,
    )
}

#[allow(clippy::too_many_arguments)] // The fixture makes each persisted timing fence explicit.
fn claim_at(
    run: u8,
    schedule: u8,
    key: ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
    scheduled_for: i64,
    claimed_at: i64,
    next_due_at: i64,
    expires_at: i64,
) -> SchedulerRunClaimV1 {
    SchedulerRunClaimV1::new(
        ScheduleRunLeaseV1::new(
            JobRunIdV1::new([run; 16]).expect("run ID"),
            ScheduleIdV1::new([schedule; 16]).expect("schedule ID"),
            ScheduleRevisionV1::new(1).expect("revision"),
            1,
            UtcMillisV1::new(expires_at),
        )
        .expect("lease"),
        UtcMillisV1::new(scheduled_for),
        UtcMillisV1::new(claimed_at),
        UtcMillisV1::new(next_due_at),
        key,
        policy,
        [run; 16],
        [run; 32],
    )
    .expect("claim request")
}

fn policy(overlap: OverlapPolicyV1) -> SchedulePolicyV1 {
    SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        overlap,
        MisfirePolicyV1::Skip,
        RetryPolicyV1::new(1, 1_000).expect("retry policy"),
        30_000,
        0,
    )
    .expect("schedule policy")
}

fn fixed_delay_policy() -> SchedulePolicyV1 {
    SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedDelay {
            delay_millis: 60_000,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::FireOnce,
        RetryPolicyV1::new(1, 1_000).expect("retry policy"),
        30_000,
        0,
    )
    .expect("fixed-delay policy")
}

fn config_change(
    id: u8,
    revision: u64,
    contract_revision: u32,
    key: ConcurrencyKeyV1,
    policy: SchedulePolicyV1,
    next_due_at: i64,
    updated_at: i64,
) -> SchedulerScheduleUpsertV1 {
    let kind = JobKindV1::new("platform".into(), "maintenance".into(), 1).expect("job kind");
    let binding = JobContractBindingV1::new(
        kind,
        "platform.maintenance".into(),
        contract_revision,
        [7; 32],
    )
    .expect("contract binding");
    let spec = ScheduleSpecV1::new(
        ScheduleIdV1::new([id; 16]).expect("schedule ID"),
        ScheduleRevisionV1::new(revision).expect("revision"),
        binding,
        OpaqueScheduleScopeV1::new("scope:technical".into()).expect("scope"),
        key,
        true,
        policy,
    );
    SchedulerScheduleUpsertV1::new(
        spec,
        UtcMillisV1::new(next_due_at),
        UtcMillisV1::new(updated_at),
    )
}

fn pending_fire(claim: SchedulerRunClaimV1) -> SchedulerPendingFireV1 {
    let recorded_at = UtcMillisV1::new(claim.scheduled_for().value().max(CLAIMED_AT + 1));
    SchedulerPendingFireV1::new(claim, recorded_at).expect("pending fire request")
}

fn key(value: &str) -> ConcurrencyKeyV1 {
    ConcurrencyKeyV1::new(value.to_owned()).expect("opaque concurrency key")
}

fn assert_one_claim_won(
    first: &Result<(), SchedulerRunClaimErrorV1>,
    second: &Result<(), SchedulerRunClaimErrorV1>,
) {
    assert!(
        (first.is_ok() && *second == Err(SchedulerRunClaimErrorV1::ConcurrencyExhausted))
            || (second.is_ok() && *first == Err(SchedulerRunClaimErrorV1::ConcurrencyExhausted))
    );
}

async fn due_at(pool: &PgPool, id: u8) -> i64 {
    sqlx::query("SELECT next_due_at_unix_ms FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1")
        .bind(vec![id; 16])
        .fetch_one(pool)
        .await
        .expect("read schedule due")
        .get("next_due_at_unix_ms")
}

async fn pending_count(pool: &PgPool, id: u8) -> i64 {
    sqlx::query("SELECT COUNT(*) AS count FROM makosh_platform.scheduler_pending_fires WHERE schedule_id = $1")
        .bind(vec![id; 16])
        .fetch_one(pool)
        .await
        .expect("count pending fires")
        .get("count")
}

async fn pending_due_at(pool: &PgPool, id: u8) -> i64 {
    sqlx::query("SELECT scheduled_for_unix_ms FROM makosh_platform.scheduler_pending_fires WHERE schedule_id = $1")
        .bind(vec![id; 16])
        .fetch_one(pool)
        .await
        .expect("load coalesced pending fire")
        .get("scheduled_for_unix_ms")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
