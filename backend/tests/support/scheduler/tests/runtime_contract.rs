use makosh_runtime_protocol::{
    v1::{
        SchedulerRuntimeConfigurationV1, SchedulerRuntimeControlRequestV1,
        SchedulerRuntimeControlResponseV1, SchedulerRuntimeDispatchPublisherBindingV1,
        SchedulerRuntimeReceiptConsumerBindingV1, SchedulerRuntimeReceiptKindV1,
        SchedulerRuntimeScheduleControlBindingV1, SchedulerRuntimeScheduleControlGrantV1,
        SchedulerRuntimeStateV1, SchedulerRuntimeStatusV1, SchedulerRuntimeStorageBindingV1,
        SchedulerScheduleUpsertOutcomeV1, UpsertSchedulerScheduleRequestV1,
        UpsertSchedulerScheduleResponseV1,
        scheduler_runtime_control_request_v1::Operation as RequestOperation,
        scheduler_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::scheduler::{
        validate_scheduler_runtime_configuration, validate_scheduler_runtime_control_request,
        validate_scheduler_runtime_control_response,
    },
};
use makosh_scheduler_protocol::{
    MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, SchedulePolicyV1, ScheduleTriggerV1,
};

#[test]
fn scheduler_runtime_contract_requires_only_fenced_non_secret_dependencies() {
    assert!(validate_scheduler_runtime_configuration(&configuration()).is_ok());
    let mut schedule_control = configuration();
    schedule_control.schedule_control = Some(schedule_control_binding());
    schedule_control.schedule_control_grants = vec![schedule_control_grant()];
    assert!(validate_scheduler_runtime_configuration(&schedule_control).is_ok());
    assert!(
        validate_scheduler_runtime_control_response(&SchedulerRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(SchedulerRuntimeStatusV1 {
                state: SchedulerRuntimeStateV1::Ready as i32,
                runtime_generation: 4,
                grant_epoch: 7,
                storage_generation: 5,
                vault_runtime_generation: 8,
                event_credential_revision: 2,
                blocker_code: String::new(),
            })),
            error_code: String::new(),
        })
        .is_ok()
    );
    assert!(
        validate_scheduler_runtime_control_request(&SchedulerRuntimeControlRequestV1 {
            operation: Some(RequestOperation::UpsertSchedule(schedule_upsert())),
        })
        .is_ok()
    );
    assert!(
        validate_scheduler_runtime_control_response(&SchedulerRuntimeControlResponseV1 {
            result: Some(ResponseResult::UpsertSchedule(
                UpsertSchedulerScheduleResponseV1 {
                    outcome: SchedulerScheduleUpsertOutcomeV1::Inserted as i32,
                    schedule_revision: 4,
                }
            )),
            error_code: String::new(),
        })
        .is_ok()
    );
}

#[test]
fn scheduler_runtime_contract_rejects_secret_bearing_or_unbounded_configuration() {
    assert_invalid_configuration(|value| {
        value.nats_endpoint = "nats://user:password@broker".to_owned()
    });
    assert_invalid_configuration(|value| value.dispatch_batch_limit = 257);
    assert_invalid_configuration(|value| fixture_binding(value).role_epoch = 0);
    assert_invalid_configuration(|value| value.logical_owner_id = "storage".to_owned());
    assert_invalid_configuration(|value| value.runtime_instance_id.clear());
    assert_invalid_configuration(|value| {
        fixture_binding(value).storage_bundle_digest = vec![0; 32]
    });
    assert_invalid_configuration(|value| value.dispatch_publishers.clear());
    assert_invalid_configuration(|value| {
        value.dispatch_publishers[0].subject = "makosh.event.v1.mail.sync_job.v1".to_owned();
    });
    assert_invalid_configuration(|value| {
        value.receipt_consumers[1].filter_subject = "makosh.ack.v1.mail.job_receipt.v1".to_owned();
    });
    assert_invalid_configuration(|value| {
        value.receipt_consumers.pop();
    });
    assert_invalid_configuration(|value| {
        value.schedule_control = Some(schedule_control_binding());
    });
    assert_invalid_configuration(|value| {
        value.schedule_control_grants = vec![schedule_control_grant()];
    });
    assert_invalid_configuration(|value| {
        value.schedule_control = Some(schedule_control_binding());
        value.schedule_control_grants = vec![schedule_control_grant()];
        value.schedule_control_grants[0].source_owner = "mail".to_owned();
    });

    assert_extended_receipts_are_allowed();
    assert_secret_status_is_rejected();
    assert_invalid_schedule_schema_is_rejected();
}

fn assert_extended_receipts_are_allowed() {
    let mut extended = configuration();
    extended.receipt_consumers.extend([
        receipt_consumer(
            SchedulerRuntimeReceiptKindV1::Acceptance,
            "scheduler_receipt_acceptance_second",
            "makosh.ack.v1.documents.job_receipt.v1",
        ),
        receipt_consumer(
            SchedulerRuntimeReceiptKindV1::Terminal,
            "scheduler_receipt_terminal_second",
            "makosh.result.v1.documents.job_receipt.v1",
        ),
    ]);
    assert!(validate_scheduler_runtime_configuration(&extended).is_ok());
}

fn assert_secret_status_is_rejected() {
    assert!(
        validate_scheduler_runtime_control_response(&SchedulerRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(SchedulerRuntimeStatusV1 {
                state: SchedulerRuntimeStateV1::Ready as i32,
                runtime_generation: 4,
                grant_epoch: 7,
                storage_generation: 5,
                vault_runtime_generation: 8,
                event_credential_revision: 2,
                blocker_code: "credential_unavailable".to_owned(),
            })),
            error_code: String::new(),
        })
        .is_err()
    );
}

fn assert_invalid_schedule_schema_is_rejected() {
    let mut invalid_schedule = schedule_upsert();
    invalid_schedule.contract_schema_sha256 = vec![0; 32];
    assert!(
        validate_scheduler_runtime_control_request(&SchedulerRuntimeControlRequestV1 {
            operation: Some(RequestOperation::UpsertSchedule(invalid_schedule)),
        })
        .is_err()
    );
}

fn assert_invalid_configuration(mutator: impl FnOnce(&mut SchedulerRuntimeConfigurationV1)) {
    let mut invalid = configuration();
    mutator(&mut invalid);
    assert!(validate_scheduler_runtime_configuration(&invalid).is_err());
}

fn fixture_binding(
    configuration: &mut SchedulerRuntimeConfigurationV1,
) -> &mut SchedulerRuntimeStorageBindingV1 {
    configuration
        .storage_binding
        .as_mut()
        .expect("fixture binding")
}

fn schedule_upsert() -> UpsertSchedulerScheduleRequestV1 {
    UpsertSchedulerScheduleRequestV1 {
        schedule_id: vec![1; 16],
        schedule_revision: 4,
        job_owner: "mail".to_owned(),
        job_name: "sync_job".to_owned(),
        job_major: 1,
        contract_name: "mail.sync_job".to_owned(),
        contract_revision: 3,
        contract_schema_sha256: vec![7; 32],
        scope_id: "account:opaque_1".to_owned(),
        concurrency_key: "account:opaque_1".to_owned(),
        enabled: true,
        policy_canonical_bytes: schedule_policy().canonical_bytes(),
        next_due_at_unix_millis: 1_000,
        updated_at_unix_millis: 1_000,
    }
}

fn schedule_policy() -> SchedulePolicyV1 {
    SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::Skip,
        RetryPolicyV1::new(2, 1_000).expect("bounded retry policy"),
        30_000,
        0,
    )
    .expect("valid scheduler policy")
}

fn configuration() -> SchedulerRuntimeConfigurationV1 {
    SchedulerRuntimeConfigurationV1 {
        storage_binding: Some(SchedulerRuntimeStorageBindingV1 {
            database_id: "makosh_scheduler".to_owned(),
            pgbouncer_host: "127.0.0.1".to_owned(),
            pgbouncer_port: 6432,
            runtime_principal: "scheduler_runtime".to_owned(),
            storage_generation: 5,
            credential_revision: 3,
            storage_instance_id: "storage_main".to_owned(),
            owner: "scheduler".to_owned(),
            role_epoch: 6,
            pool_alias: "runtime_scheduler_registration_4".to_owned(),
            max_connections: 8,
            statement_timeout_millis: 30_000,
            storage_bundle_revision: 1,
            storage_bundle_digest: vec![9; 32],
        }),
        vault_instance_id: "vault_main".to_owned(),
        vault_runtime_generation: 8,
        vault_hpke_public_key_x25519: vec![7; 32],
        runtime_instance_id: "scheduler_runtime".to_owned(),
        logical_owner_id: "scheduler".to_owned(),
        nats_endpoint: "nats://127.0.0.1:4222".to_owned(),
        event_credential_revision: 2,
        dispatch_batch_limit: 32,
        receipt_batch_limit: 32,
        reconcile_interval_millis: 1_000,
        dispatch_publishers: vec![SchedulerRuntimeDispatchPublisherBindingV1 {
            subject: "makosh.command.v1.mail.sync_job.v1".to_owned(),
        }],
        receipt_consumers: vec![
            receipt_consumer(
                SchedulerRuntimeReceiptKindV1::Acceptance,
                "scheduler_receipt_acceptance",
                "makosh.ack.v1.mail.job_receipt.v1",
            ),
            receipt_consumer(
                SchedulerRuntimeReceiptKindV1::Terminal,
                "scheduler_receipt_terminal",
                "makosh.result.v1.mail.job_receipt.v1",
            ),
        ],
        schedule_control: None,
        schedule_control_grants: Vec::new(),
    }
}

fn schedule_control_binding() -> SchedulerRuntimeScheduleControlBindingV1 {
    SchedulerRuntimeScheduleControlBindingV1 {
        stream_name: "MAKOSH_COMMAND_V1".to_owned(),
        durable_name: "scheduler_schedule_control".to_owned(),
        filter_subject: "makosh.command.v1.scheduler.schedule_control.v1".to_owned(),
        ack_wait_millis: 30_000,
        max_deliver: 8,
        max_ack_pending: 32,
        result_subject: "makosh.result.v1.scheduler.schedule_control.v1".to_owned(),
        command_contract_revision: 1,
        command_schema_sha256: vec![7; 32],
        result_contract_revision: 1,
        result_schema_sha256: vec![8; 32],
    }
}

fn schedule_control_grant() -> SchedulerRuntimeScheduleControlGrantV1 {
    SchedulerRuntimeScheduleControlGrantV1 {
        source_module_id: "communication_delayed_delivery.runtime.v1".to_owned(),
        source_runtime_instance_id: vec![4; 16],
        source_runtime_generation: 3,
        source_grant_epoch: 5,
        source_owner: "communication_delayed_delivery".to_owned(),
        job_owner: "communication_delayed_delivery".to_owned(),
        job_name: "execute".to_owned(),
        job_major: 1,
        contract_name: "communication_delayed_delivery.execute".to_owned(),
        contract_revision: 1,
        contract_schema_sha256: vec![6; 32],
    }
}

fn receipt_consumer(
    kind: SchedulerRuntimeReceiptKindV1,
    durable_name: &str,
    filter_subject: &str,
) -> SchedulerRuntimeReceiptConsumerBindingV1 {
    let stream_name = match kind {
        SchedulerRuntimeReceiptKindV1::Acceptance => "MAKOSH_ACK_V1",
        SchedulerRuntimeReceiptKindV1::Terminal => "MAKOSH_RESULT_V1",
        SchedulerRuntimeReceiptKindV1::Unspecified => unreachable!(),
    };
    SchedulerRuntimeReceiptConsumerBindingV1 {
        kind: kind as i32,
        stream_name: stream_name.to_owned(),
        durable_name: durable_name.to_owned(),
        filter_subject: filter_subject.to_owned(),
        ack_wait_millis: 30_000,
        max_deliver: 8,
        max_ack_pending: 32,
    }
}
