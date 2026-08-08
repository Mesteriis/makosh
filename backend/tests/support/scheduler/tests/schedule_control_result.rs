use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::decode_envelope_v1,
};
use makosh_scheduler::{
    SchedulerDispatchIdentityV1, SchedulerScheduleControlContractV1,
    SchedulerScheduleControlResultBuildErrorV1, build_schedule_control_result_envelope_v1,
};
use makosh_scheduler_protocol::v1::{
    EnsureOneShotScheduleV1, JobKindV1, SchedulerScheduleControlCommandV1,
    SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1,
    scheduler_schedule_control_command_v1::Operation,
};
use prost::Message;
use prost_types::Timestamp;

#[test]
fn schedule_control_result_preserves_command_lineage_and_uses_scheduler_fence() {
    let command = command_record();
    let payload = successful_payload();
    let source =
        SchedulerDispatchIdentityV1::new("scheduler".to_owned(), [3; 16], 4).expect("source");
    let contract = SchedulerScheduleControlContractV1::new(2, [7; 32]).expect("contract");

    let envelope = build_schedule_control_result_envelope_v1(
        &command,
        payload.clone(),
        [5; 16],
        UtcMillisV1::new(2_500),
        &source,
        &contract,
    )
    .expect("result envelope");
    let decoded = decode_envelope_v1(&envelope.encode_to_vec()).expect("canonical envelope");

    assert_eq!(decoded.message_id, vec![5; 16]);
    assert_eq!(decoded.partition_key, vec![8; 16]);
    assert_eq!(decoded.correlation_id, vec![6; 16]);
    assert_eq!(decoded.causation_message_id, vec![9; 16]);
    assert_eq!(
        decoded.source.as_ref().expect("source").module_id,
        "scheduler"
    );
    assert_eq!(
        decoded.source_fence.as_ref().expect("source fence").kind,
        FenceKindV1::RuntimeLease as i32
    );
    let Some(Semantics::Result(metadata)) = decoded.semantics.as_ref() else {
        panic!("result semantics");
    };
    assert_eq!(metadata.command_id, vec![6; 16]);
    assert_eq!(metadata.command_message_id, vec![9; 16]);
    assert_eq!(metadata.outcome, ResultOutcomeV1::Succeeded as i32);
    assert_eq!(
        SchedulerScheduleControlResultV1::decode(decoded.payload.as_slice())
            .expect("typed payload"),
        payload
    );
}

#[test]
fn schedule_control_result_rejects_zero_message_id_and_invalid_payload() {
    let command = command_record();
    let source =
        SchedulerDispatchIdentityV1::new("scheduler".to_owned(), [3; 16], 4).expect("source");
    let contract = SchedulerScheduleControlContractV1::new(2, [7; 32]).expect("contract");

    assert_eq!(
        build_schedule_control_result_envelope_v1(
            &command,
            successful_payload(),
            [0; 16],
            UtcMillisV1::new(2_500),
            &source,
            &contract,
        ),
        Err(SchedulerScheduleControlResultBuildErrorV1::InvalidMessageId)
    );

    let mut invalid = successful_payload();
    invalid.error_code = "unexpected_error".to_owned();
    assert_eq!(
        build_schedule_control_result_envelope_v1(
            &command,
            invalid,
            [5; 16],
            UtcMillisV1::new(2_500),
            &source,
            &contract,
        ),
        Err(SchedulerScheduleControlResultBuildErrorV1::InvalidPayload)
    );
}

fn successful_payload() -> SchedulerScheduleControlResultV1 {
    SchedulerScheduleControlResultV1 {
        operation_id: vec![6; 16],
        schedule_id: vec![8; 16],
        schedule_revision: 1,
        outcome: SchedulerScheduleControlOutcomeV1::Ensured.into(),
        error_code: String::new(),
    }
}

fn command_record() -> OutboxRecordV1 {
    OutboxRecordV1::accept(command_envelope().encode_to_vec()).expect("command record")
}

fn command_envelope() -> DurableEnvelopeV1 {
    let module = "communication_delayed_delivery.runtime.v1";
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![9; 16],
        contract: Some(ContractRefV1 {
            owner: "scheduler".to_owned(),
            name: "schedule_control".to_owned(),
            major: 1,
            revision: 2,
            schema_sha256: vec![7; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: module.to_owned(),
            runtime_instance_id: vec![4; 16],
            runtime_generation: 3,
        }),
        recorded_at: Some(Timestamp {
            seconds: 1,
            nanos: 0,
        }),
        partition_key: vec![8; 16],
        causation_message_id: Vec::new(),
        correlation_id: vec![6; 16],
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module.into(),
            actor_id: module.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::GrantEpoch.into(),
            scope_id: module.as_bytes().to_vec(),
            epoch: 5,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: vec![6; 16],
            target_capability: "scheduler_schedule_control".to_owned(),
            idempotency_key: vec![6; 16],
            deadline: Some(Timestamp {
                seconds: 60,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: SchedulerScheduleControlCommandV1 {
            operation_id: vec![6; 16],
            operation: Some(Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
                schedule_id: vec![8; 16],
                schedule_revision: 1,
                job_kind: Some(JobKindV1 {
                    owner: "communication_delayed_delivery".to_owned(),
                    name: "execute".to_owned(),
                    major: 1,
                }),
                job_contract_revision: 1,
                job_schema_sha256: vec![5; 32],
                scope_id: "delayed-operation-1".to_owned(),
                concurrency_key: "delayed-operation-1".to_owned(),
                due_at_unix_millis: 2_000,
                deadline_millis: 30_000,
                max_attempts: 3,
                retry_base_backoff_millis: 1_000,
            })),
        }
        .encode_to_vec(),
    }
}
