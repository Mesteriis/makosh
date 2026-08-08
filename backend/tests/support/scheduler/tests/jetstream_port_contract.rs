use makosh_events_protocol::RuntimeNatsJwtCredentialV1;
use makosh_runtime_protocol::v1::{
    SchedulerRuntimeReceiptConsumerBindingV1, SchedulerRuntimeReceiptKindV1,
    SchedulerRuntimeScheduleControlBindingV1,
};
use makosh_scheduler_jetstream::{
    SchedulerJetStreamReceiptPortErrorV1, SchedulerJetStreamReceiptPortV1,
    SchedulerJetStreamScheduleControlPortErrorV1, SchedulerJetStreamScheduleControlPortV1,
};
use nats_jwt::KeyPair;

#[tokio::test]
async fn scheduler_jetstream_port_rejects_an_expired_jwt_before_connecting() {
    let result = SchedulerJetStreamReceiptPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(1),
        &binding(),
    )
    .await;

    assert!(matches!(
        result,
        Err(SchedulerJetStreamReceiptPortErrorV1::ExpiredCredential)
    ));
}

#[tokio::test]
async fn schedule_control_port_rejects_expired_credentials_and_wildcard_bindings() {
    let expired = SchedulerJetStreamScheduleControlPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(1),
        &schedule_control_binding(),
    )
    .await;
    assert!(matches!(
        expired,
        Err(SchedulerJetStreamScheduleControlPortErrorV1::ExpiredCredential)
    ));

    let mut wildcard = schedule_control_binding();
    wildcard.filter_subject = "makosh.command.v1.scheduler.>".to_owned();
    let invalid = SchedulerJetStreamScheduleControlPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(u64::MAX),
        &wildcard,
    )
    .await;
    assert!(matches!(
        invalid,
        Err(SchedulerJetStreamScheduleControlPortErrorV1::InvalidBinding)
    ));
}

#[tokio::test]
async fn scheduler_jetstream_port_rejects_a_binding_before_connecting() {
    let mut invalid = binding();
    invalid.filter_subject = "makosh.ack.v1.>".to_owned();
    let result = SchedulerJetStreamReceiptPortV1::connect(
        "nats://127.0.0.1:4222",
        credential(u64::MAX),
        &invalid,
    )
    .await;

    assert!(matches!(
        result,
        Err(SchedulerJetStreamReceiptPortErrorV1::InvalidBinding)
    ));
}

fn credential(expires_at_unix_seconds: u64) -> RuntimeNatsJwtCredentialV1 {
    let key = KeyPair::new_user();
    RuntimeNatsJwtCredentialV1::new(
        "test-jwt".to_owned(),
        key.seed().expect("user seed"),
        key.public_key(),
        expires_at_unix_seconds,
    )
    .expect("runtime credential")
}

fn binding() -> SchedulerRuntimeReceiptConsumerBindingV1 {
    SchedulerRuntimeReceiptConsumerBindingV1 {
        kind: SchedulerRuntimeReceiptKindV1::Acceptance as i32,
        stream_name: "MAKOSH_ACK_V1".to_owned(),
        durable_name: "scheduler_receipt_acceptance".to_owned(),
        filter_subject: "makosh.ack.v1.mail.job_receipt.v1".to_owned(),
        ack_wait_millis: 30_000,
        max_deliver: 8,
        max_ack_pending: 32,
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
