use makosh_events_protocol::v1::{
    ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
    SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
};
use makosh_scheduler::{
    SchedulerApprovedJobV1, SchedulerScheduleControlAdmissionErrorV1,
    SchedulerScheduleControlContractV1, SchedulerScheduleControlGrantV1,
    SchedulerScheduleControlOperationV1, admit_schedule_control_command_v1,
};
use makosh_scheduler_protocol::{
    JobContractBindingV1, JobKindV1 as CanonicalJobKindV1,
    v1::{
        EnsureOneShotScheduleV1, JobKindV1, SchedulerScheduleControlCommandV1,
        scheduler_schedule_control_command_v1::Operation,
    },
};
use prost::Message;
use prost_types::Timestamp;

#[test]
fn schedule_control_admission_requires_exact_contract_runtime_and_grant_fences() {
    let contract = SchedulerScheduleControlContractV1::new(1, [7; 32]).expect("contract");
    let grant = grant();
    let bytes = envelope(command("communication_delayed_delivery"), 3, 5).encode_to_vec();
    let admitted =
        admit_schedule_control_command_v1(&bytes, &contract, std::slice::from_ref(&grant))
            .expect("admitted command");
    assert_eq!(admitted.operation_id(), &[6; 16]);
    assert!(matches!(
        admitted.operation(),
        SchedulerScheduleControlOperationV1::Ensure(_)
    ));

    let stale = envelope(command("communication_delayed_delivery"), 4, 5).encode_to_vec();
    assert_eq!(
        admit_schedule_control_command_v1(&stale, &contract, std::slice::from_ref(&grant)),
        Err(SchedulerScheduleControlAdmissionErrorV1::StaleFence)
    );

    let foreign = envelope(command("mail"), 3, 5).encode_to_vec();
    assert_eq!(
        admit_schedule_control_command_v1(&foreign, &contract, &[grant]),
        Err(SchedulerScheduleControlAdmissionErrorV1::ForeignJobKind)
    );
}

fn grant() -> SchedulerScheduleControlGrantV1 {
    let kind = CanonicalJobKindV1::new(
        "communication_delayed_delivery".to_owned(),
        "execute".to_owned(),
        1,
    )
    .expect("job kind");
    let binding = JobContractBindingV1::new(
        kind,
        "communication_delayed_delivery.execute".to_owned(),
        1,
        [5; 32],
    )
    .expect("binding");
    let approved =
        SchedulerApprovedJobV1::new("communication_delayed_delivery".to_owned(), binding)
            .expect("approved job");
    SchedulerScheduleControlGrantV1::new(
        "communication_delayed_delivery.runtime.v1".to_owned(),
        [4; 16],
        3,
        5,
        approved,
    )
    .expect("grant")
}

fn command(owner: &str) -> SchedulerScheduleControlCommandV1 {
    SchedulerScheduleControlCommandV1 {
        operation_id: vec![6; 16],
        operation: Some(Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
            schedule_id: vec![8; 16],
            schedule_revision: 1,
            job_kind: Some(JobKindV1 {
                owner: owner.to_owned(),
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
}

fn envelope(
    command: SchedulerScheduleControlCommandV1,
    runtime_generation: u64,
    grant_epoch: u64,
) -> DurableEnvelopeV1 {
    let module = "communication_delayed_delivery.runtime.v1";
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![9; 16],
        contract: Some(ContractRefV1 {
            owner: "scheduler".to_owned(),
            name: "schedule_control".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: module.to_owned(),
            runtime_instance_id: vec![4; 16],
            runtime_generation,
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
            epoch: grant_epoch,
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
        payload: command.encode_to_vec(),
    }
}
