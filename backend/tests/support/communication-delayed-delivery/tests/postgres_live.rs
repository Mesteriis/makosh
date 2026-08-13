//! Disposable PostgreSQL proof for the Delayed Delivery durable lifecycle.

use makosh_communication_delayed_delivery_core::{
    DelayedDeliveryDraftV1, DelayedDeliveryStateV1, prepare_delayed_delivery_v1,
};
use makosh_communication_delayed_delivery_persistence::{
    ApplySchedulerResultOutcomeV1, ApplySchedulerResultV1, ClaimDueExecutionOutcomeV1,
    ClaimDueExecutionV1, CreateDelayedDeliveryOperationOutcomeV1, CreateDelayedDeliveryOperationV1,
    DelayedDeliveryBodyCleanupReasonV1, DelayedDeliveryBodyReceiptV1,
    DelayedDeliveryDurableMessageV1, DelayedDeliveryPersistenceConformanceV1,
    DelayedDeliveryPersistenceErrorV1, MarkDeliveryFailedV1, RequestDelayedDeliveryCancellationV1,
    SchedulerExecutionFenceV1, SchedulerScheduleResultV1,
    schema::communication_delayed_delivery_storage_bundle_v1,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

const POSTGRES_URL: &str = "MAKOSH_COMMUNICATION_DELAYED_DELIVERY_POSTGRES_URL";
const OWNER: &str = "owner-1";
const CREATED_AT: u64 = 1_000;
const DELIVER_AT: u64 = 6_000;

#[tokio::test]
#[ignore = "requires the disposable Delayed Delivery PostgreSQL contour"]
async fn durable_lifecycle_survives_restart_and_fences_duplicates_and_cancel_races() {
    let database_url = required(POSTGRES_URL);
    let admin = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("connect disposable Delayed Delivery PostgreSQL");
    install_schema(&admin).await;
    install_nobypass_rls_role(&admin).await;
    let persistence = connect_nobypass(&database_url).await;
    persistence
        .verify_storage_ready()
        .await
        .expect("verify Delayed Delivery storage");

    let create = create_command(1, 5);
    assert_eq!(
        persistence.create_operation(&create).await,
        Ok(CreateDelayedDeliveryOperationOutcomeV1::Created { state_revision: 1 })
    );
    assert_eq!(
        persistence.create_operation(&create).await,
        Ok(CreateDelayedDeliveryOperationOutcomeV1::Existing { state_revision: 1 })
    );
    let mut conflicting_create = create.clone();
    conflicting_create.body_receipt.custody_proof = vec![0xEE];
    assert_eq!(
        persistence.create_operation(&conflicting_create).await,
        Err(DelayedDeliveryPersistenceErrorV1::Conflict)
    );
    assert_eq!(
        persistence
            .pending_scheduler_commands(OWNER, 8)
            .await
            .expect("read Scheduler command outbox")
            .len(),
        1
    );

    let ensured = scheduler_result(
        1,
        8,
        SchedulerScheduleResultV1::Ensured {
            schedule_revision: 1,
        },
    );
    let applied = persistence
        .apply_scheduler_result(&ensured)
        .await
        .expect("apply Scheduler ensure");
    assert!(matches!(
        applied,
        ApplySchedulerResultOutcomeV1::Applied(ref status)
            if status.state == DelayedDeliveryStateV1::Scheduled
                && status.state_revision == 2
    ));
    assert!(matches!(
        persistence.apply_scheduler_result(&ensured).await,
        Ok(ApplySchedulerResultOutcomeV1::Duplicate(ref status))
            if status.state == DelayedDeliveryStateV1::Scheduled
    ));
    let mut conflicting_result = ensured.clone();
    conflicting_result.envelope_sha256 = [0xEF; 32];
    assert_eq!(
        persistence
            .apply_scheduler_result(&conflicting_result)
            .await,
        Err(DelayedDeliveryPersistenceErrorV1::Conflict)
    );
    assert_eq!(
        persistence
            .request_cancellation(&RequestDelayedDeliveryCancellationV1 {
                logical_owner_id: OWNER.to_owned(),
                delayed_operation_id: [1; 16],
                expected_revision: 1,
                scheduler_command: durable_message(9, "scheduler.schedule.command.v1"),
                requested_at_unix_millis: 6_100,
            })
            .await,
        Err(DelayedDeliveryPersistenceErrorV1::StaleRevision)
    );

    let due = due_command(1, 10);
    let claim = match persistence
        .claim_due_execution(&due)
        .await
        .expect("claim due execution")
    {
        ClaimDueExecutionOutcomeV1::Claimed(claim) => claim,
        ClaimDueExecutionOutcomeV1::Duplicate(_) => panic!("first due command must claim"),
    };
    assert_eq!(claim.fence, due.fence);
    assert!(matches!(
        persistence.claim_due_execution(&due).await,
        Ok(ClaimDueExecutionOutcomeV1::Duplicate(ref duplicate)) if duplicate == &claim
    ));
    let mut conflicting_due = due.clone();
    conflicting_due.fence.lease_epoch = 2;
    assert_eq!(
        persistence.claim_due_execution(&conflicting_due).await,
        Err(DelayedDeliveryPersistenceErrorV1::Conflict)
    );
    persistence
        .mark_delivery_failed(&MarkDeliveryFailedV1 {
            claim: claim.clone(),
            error_code: 1,
            terminal_receipt: durable_message(11, "scheduler.job_run.result.v1"),
            failed_at_unix_millis: 7_100,
        })
        .await
        .expect("persist terminal failure");

    assert_cancel_too_late_race(&persistence).await;
    let cleanup = persistence
        .next_body_cleanup(OWNER, 7_100)
        .await
        .expect("read terminal body cleanup")
        .expect("terminal failure must enqueue body cleanup");
    assert_eq!(cleanup.delayed_operation_id, [1; 16]);
    assert_eq!(
        cleanup.reason,
        DelayedDeliveryBodyCleanupReasonV1::DeliveryRejected
    );
    assert_eq!(cleanup.attempt_count, 0);
    persistence
        .reschedule_body_cleanup(OWNER, &[1; 16], 0, 7_500, 7_100)
        .await
        .expect("persist body cleanup retry");
    assert_cancelled_cleanup_enqueued(&persistence).await;
    drop(persistence);

    let reopened = connect_nobypass(&database_url).await;
    let terminal = reopened
        .status(OWNER, &[1; 16])
        .await
        .expect("read terminal state after reconnect");
    assert_eq!(terminal.state, DelayedDeliveryStateV1::Failed);
    assert_eq!(terminal.error_code, Some(1));
    let transitions = reopened
        .client_realtime_window(OWNER, None, 16)
        .await
        .expect("replay state transitions after reconnect");
    assert!(
        transitions.iter().any(|transition| {
            transition.delayed_operation_id == [1; 16]
                && transition.state == DelayedDeliveryStateV1::Failed
        }),
        "terminal transition must remain replayable after reconnect"
    );
    assert_eq!(
        reopened
            .pending_scheduler_receipts(OWNER, 8)
            .await
            .expect("read Scheduler receipt outbox")
            .len(),
        2
    );
    assert_eq!(
        reopened
            .next_body_cleanup(OWNER, 7_499)
            .await
            .expect("read cleanup before retry deadline"),
        None
    );
    let retried_cleanup = reopened
        .next_body_cleanup(OWNER, 7_500)
        .await
        .expect("read cleanup after reconnect")
        .expect("cleanup retry must survive reconnect");
    assert_eq!(retried_cleanup.delayed_operation_id, [1; 16]);
    assert_eq!(retried_cleanup.attempt_count, 1);
    reopened
        .complete_body_cleanup(OWNER, &[1; 16], 7_600)
        .await
        .expect("complete durable body cleanup");
    let cancelled_cleanup = reopened
        .next_body_cleanup(OWNER, 7_601)
        .await
        .expect("read cancelled body cleanup")
        .expect("successful cancellation must enqueue body cleanup");
    assert_eq!(cancelled_cleanup.delayed_operation_id, [3; 16]);
    assert_eq!(
        cancelled_cleanup.reason,
        DelayedDeliveryBodyCleanupReasonV1::DeliveryCancelled
    );
    reopened
        .complete_body_cleanup(OWNER, &[3; 16], 7_602)
        .await
        .expect("complete cancelled body cleanup");
    assert_eq!(
        reopened
            .next_body_cleanup(OWNER, 7_602)
            .await
            .expect("read completed cleanup queue"),
        None
    );
    assert_owner_rls(&admin, &database_url).await;
}

async fn assert_cancel_too_late_race(
    persistence: &makosh_communication_delayed_delivery_persistence::CommunicationDelayedDeliveryPersistenceV1,
) {
    persistence
        .create_operation(&create_command(2, 20))
        .await
        .expect("create cancellation-race operation");
    persistence
        .apply_scheduler_result(&scheduler_result(
            2,
            21,
            SchedulerScheduleResultV1::Ensured {
                schedule_revision: 1,
            },
        ))
        .await
        .expect("schedule cancellation-race operation");
    let cancelling = persistence
        .request_cancellation(&RequestDelayedDeliveryCancellationV1 {
            logical_owner_id: OWNER.to_owned(),
            delayed_operation_id: [2; 16],
            expected_revision: 2,
            scheduler_command: durable_message(22, "scheduler.schedule.command.v1"),
            requested_at_unix_millis: 6_200,
        })
        .await
        .expect("request cancellation");
    assert_eq!(cancelling.state, DelayedDeliveryStateV1::CancelRequested);
    let too_late = persistence
        .apply_scheduler_result(&scheduler_result(2, 23, SchedulerScheduleResultV1::TooLate))
        .await
        .expect("apply Scheduler too-late result");
    assert!(matches!(
        too_late,
        ApplySchedulerResultOutcomeV1::Applied(ref status)
            if status.state == DelayedDeliveryStateV1::Scheduled
                && status.state_revision == 4
    ));
}

async fn assert_cancelled_cleanup_enqueued(
    persistence: &makosh_communication_delayed_delivery_persistence::CommunicationDelayedDeliveryPersistenceV1,
) {
    persistence
        .create_operation(&create_command(3, 30))
        .await
        .expect("create cancellable operation");
    persistence
        .apply_scheduler_result(&scheduler_result(
            3,
            31,
            SchedulerScheduleResultV1::Ensured {
                schedule_revision: 1,
            },
        ))
        .await
        .expect("schedule cancellable operation");
    persistence
        .request_cancellation(&RequestDelayedDeliveryCancellationV1 {
            logical_owner_id: OWNER.to_owned(),
            delayed_operation_id: [3; 16],
            expected_revision: 2,
            scheduler_command: durable_message(32, "scheduler.schedule.command.v1"),
            requested_at_unix_millis: 6_300,
        })
        .await
        .expect("request successful cancellation");
    let mut cancelled_result = scheduler_result(3, 33, SchedulerScheduleResultV1::Cancelled);
    cancelled_result.received_at_unix_millis = 7_601;
    let cancelled = persistence
        .apply_scheduler_result(&cancelled_result)
        .await
        .expect("apply Scheduler cancellation");
    assert!(matches!(
        cancelled,
        ApplySchedulerResultOutcomeV1::Applied(ref status)
            if status.state == DelayedDeliveryStateV1::Cancelled
    ));
}

fn create_command(operation_id: u8, scheduler_message_id: u8) -> CreateDelayedDeliveryOperationV1 {
    let operation = prepare_delayed_delivery_v1(
        DelayedDeliveryDraftV1 {
            delayed_operation_id: [operation_id; 16],
            delivery_operation_id: [operation_id.wrapping_add(40); 16],
            conversation_id: [operation_id.wrapping_add(80); 16],
            reply_to_message_id: None,
            body_utf8: b"private body is held by Blob".to_vec(),
            deliver_at_unix_millis: DELIVER_AT,
        },
        CREATED_AT,
    )
    .expect("prepare delayed operation");
    CreateDelayedDeliveryOperationV1 {
        logical_owner_id: OWNER.to_owned(),
        operation,
        body_receipt: DelayedDeliveryBodyReceiptV1 {
            reference_id: [operation_id.wrapping_add(20); 16],
            declared_bytes: 28,
            sha256: [operation_id.wrapping_add(30); 32],
            custody_proof: vec![operation_id.wrapping_add(31); 16],
        },
        scheduler_command: durable_message(scheduler_message_id, "scheduler.schedule.command.v1"),
        created_at_unix_millis: CREATED_AT,
    }
}

fn scheduler_result(
    operation_id: u8,
    message_id: u8,
    result: SchedulerScheduleResultV1,
) -> ApplySchedulerResultV1 {
    ApplySchedulerResultV1 {
        logical_owner_id: OWNER.to_owned(),
        delayed_operation_id: [operation_id; 16],
        message_id: [message_id; 16],
        envelope_sha256: [message_id.wrapping_add(1); 32],
        result,
        received_at_unix_millis: 6_050 + u64::from(message_id),
    }
}

fn due_command(operation_id: u8, message_id: u8) -> ClaimDueExecutionV1 {
    ClaimDueExecutionV1 {
        logical_owner_id: OWNER.to_owned(),
        delayed_operation_id: [operation_id; 16],
        command_message_id: [message_id; 16],
        command_envelope_sha256: [message_id.wrapping_add(1); 32],
        fence: SchedulerExecutionFenceV1 {
            run_id: [message_id.wrapping_add(2); 16],
            schedule_revision: 1,
            lease_epoch: 1,
            lease_expires_at_unix_millis: 20_000,
        },
        acceptance_receipt: durable_message(
            message_id.wrapping_add(3),
            "scheduler.job_run.acceptance.v1",
        ),
        claimed_at_unix_millis: 7_000,
    }
}

fn durable_message(message_id: u8, contract_kind: &'static str) -> DelayedDeliveryDurableMessageV1 {
    DelayedDeliveryDurableMessageV1 {
        message_id: [message_id; 16],
        contract_kind,
        envelope_sha256: [message_id.wrapping_add(1); 32],
        envelope_bytes: vec![message_id, message_id.wrapping_add(1)],
    }
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("CREATE SCHEMA IF NOT EXISTS makosh_data;")
        .execute(pool)
        .await
        .expect("create Delayed Delivery schema");
    for step in communication_delayed_delivery_storage_bundle_v1().steps {
        let sql =
            std::str::from_utf8(&step.forward_sql_utf8).expect("Delayed Delivery migration UTF-8");
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
            .execute(pool)
            .await
            .expect("apply Delayed Delivery migration");
    }
}

async fn connect_nobypass(
    database_url: &str,
) -> makosh_communication_delayed_delivery_persistence::CommunicationDelayedDeliveryPersistenceV1 {
    DelayedDeliveryPersistenceConformanceV1::connect_url_as_nobypass_rls_role(database_url)
        .await
        .expect("connect Delayed Delivery persistence as NOSUPERUSER NOBYPASSRLS role")
}

async fn install_nobypass_rls_role(admin: &PgPool) {
    sqlx::raw_sql(
        "CREATE ROLE makosh_delayed_delivery_rls_test \
           NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOINHERIT; \
         GRANT USAGE ON SCHEMA makosh_data TO makosh_delayed_delivery_rls_test; \
         GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA makosh_data \
           TO makosh_delayed_delivery_rls_test; \
         GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA makosh_data \
           TO makosh_delayed_delivery_rls_test;",
    )
    .execute(admin)
    .await
    .expect("create exact non-bypass Delayed Delivery RLS role");
    let attributes: (bool, bool) = sqlx::query_as(
        "SELECT rolsuper, rolbypassrls FROM pg_roles \
         WHERE rolname = 'makosh_delayed_delivery_rls_test'",
    )
    .fetch_one(admin)
    .await
    .expect("read Delayed Delivery RLS role attributes");
    assert_eq!(attributes, (false, false));
}

async fn assert_owner_rls(admin: &PgPool, database_url: &str) {
    let options = database_url
        .parse::<sqlx::postgres::PgConnectOptions>()
        .expect("parse Delayed Delivery PostgreSQL URL");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE makosh_delayed_delivery_rls_test")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(options)
        .await
        .expect("connect raw Delayed Delivery RLS role");
    for table in [
        "communication_delayed_delivery_operations",
        "communication_delayed_delivery_scheduler_inbox",
        "communication_delayed_delivery_outbox",
        "communication_delayed_delivery_scheduler_receipt_outbox",
        "communication_delayed_delivery_realtime",
        "communication_delayed_delivery_body_cleanup",
    ] {
        let row_json: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT row_to_json(source)::text FROM (\
               SELECT * FROM makosh_data.{table} \
               WHERE logical_owner_id = $1 LIMIT 1\
             ) source"
        )))
        .bind(OWNER)
        .fetch_one(admin)
        .await
        .unwrap_or_else(|error| panic!("read owner-1 {table} fixture: {error}"));

        let mut transaction = pool
            .begin()
            .await
            .expect("begin owner2 RLS read transaction");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *transaction)
            .await
            .expect("set owner2 RLS context");
        let visible: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM makosh_data.{table} WHERE logical_owner_id = $1"
        )))
        .bind(OWNER)
        .fetch_one(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 SELECT {table}: {error}"));
        assert_eq!(visible, 0, "owner2 must not see owner1 {table}");
        let updated = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE makosh_data.{table} SET logical_owner_id = logical_owner_id \
             WHERE logical_owner_id = $1"
        )))
        .bind(OWNER)
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 UPDATE {table}: {error}"))
        .rows_affected();
        assert_eq!(updated, 0, "owner2 must not update owner1 {table}");
        let deleted = sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM makosh_data.{table} WHERE logical_owner_id = $1"
        )))
        .bind(OWNER)
        .execute(&mut *transaction)
        .await
        .unwrap_or_else(|error| panic!("owner2 DELETE {table}: {error}"))
        .rows_affected();
        assert_eq!(deleted, 0, "owner2 must not delete owner1 {table}");
        transaction
            .commit()
            .await
            .expect("commit owner2 invisible DML");

        let mut insert_transaction = pool
            .begin()
            .await
            .expect("begin owner2 RLS insert transaction");
        sqlx::query("SELECT set_config('makosh.logical_owner_id', 'owner-2', true)")
            .execute(&mut *insert_transaction)
            .await
            .expect("set owner2 insert context");
        let error = sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO makosh_data.{table} OVERRIDING SYSTEM VALUE \
             SELECT (json_populate_record(NULL::makosh_data.{table}, $1::json)).*"
        )))
        .bind(&row_json)
        .execute(&mut *insert_transaction)
        .await
        .expect_err("owner2 cross-owner INSERT must fail");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("42501"),
            "owner2 INSERT into {table} must fail through RLS"
        );
    }
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}
