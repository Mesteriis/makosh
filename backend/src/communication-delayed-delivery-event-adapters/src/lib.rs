use makosh_communication_delayed_delivery_api::COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1;
use makosh_events_protocol::{
    envelope::validate_envelope_v1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_protocol::{
    SCHEDULER_RUNTIME_MODULE_ID_V1,
    v1::{
        SchedulerScheduleControlCommandV1, SchedulerScheduleControlOutcomeV1,
        SchedulerScheduleControlResultV1,
    },
    validate_scheduler_schedule_control_command_v1, validate_scheduler_schedule_control_result_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

mod due;
pub use due::{
    DecodedDelayedDeliveryDueCommandV1, DelayedDeliveryDueAdapterErrorV1,
    DelayedDeliveryDueContractV1, DelayedDeliveryDueMessageV1, DelayedDeliveryDueRuntimeContextV1,
    build_delayed_delivery_terminal_receipt_v1, decode_delayed_delivery_due_command_v1,
};

const SCHEDULER_CONTRACT_OWNER: &str = "scheduler";
const SCHEDULER_CONTRACT_NAME: &str = "schedule_control";
const SCHEDULER_TARGET_CAPABILITY: &str = "scheduler_schedule_control";
const SCHEDULER_COMMAND_KIND: &str = "scheduler.schedule.command.v1";
const MESSAGE_DOMAIN: &[u8] = b"makosh.communication-delayed-delivery.scheduler-command.v1\0";
const IDEMPOTENCY_DOMAIN: &[u8] =
    b"makosh.communication-delayed-delivery.scheduler-idempotency.v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliverySchedulerCommandContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub contract_revision: u32,
    pub contract_schema_sha256: [u8; 32],
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
    pub deadline_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliverySchedulerResultContextV1 {
    pub expected_command_message_id: [u8; 16],
    pub contract_revision: u32,
    pub contract_schema_sha256: [u8; 32],
    pub received_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliverySchedulerMessageV1 {
    pub message_id: [u8; 16],
    pub contract_kind: &'static str,
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodedSchedulerScheduleResultV1 {
    Ensured { schedule_revision: u64 },
    Cancelled,
    TooLate,
    Rejected { error_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedDelayedDeliverySchedulerResultV1 {
    pub delayed_operation_id: [u8; 16],
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub result: DecodedSchedulerScheduleResultV1,
    pub received_at_unix_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliverySchedulerAdapterErrorV1 {
    InvalidCommand,
    InvalidContext,
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongCorrelation,
    InvalidResult,
}

pub fn build_scheduler_command_v1(
    command: SchedulerScheduleControlCommandV1,
    context: &DelayedDeliverySchedulerCommandContextV1,
) -> Result<DelayedDeliverySchedulerMessageV1, DelayedDeliverySchedulerAdapterErrorV1> {
    validate_scheduler_schedule_control_command_v1(&command)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidCommand)?;
    validate_command_context(context)?;
    let operation_id = id16(&command.operation_id)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidCommand)?;
    let message_id = derived_id(MESSAGE_DOMAIN, &operation_id);
    let recorded_at = timestamp(context.recorded_at_unix_seconds, context.recorded_at_nanos)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(contract(
            context.contract_revision,
            context.contract_schema_sha256,
        )),
        source: Some(SourceRefV1 {
            module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(recorded_at),
        partition_key: operation_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: operation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1
                .as_bytes()
                .to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::GrantEpoch as i32,
            scope_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1
                .as_bytes()
                .to_vec(),
            epoch: context.grant_epoch,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: SCHEDULER_TARGET_CAPABILITY.to_owned(),
            idempotency_key: idempotency_key(&context.logical_owner_id, &operation_id),
            deadline: Some(Timestamp {
                seconds: context.deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: command.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidEnvelope)?;
    let envelope_bytes = envelope.encode_to_vec();
    Ok(DelayedDeliverySchedulerMessageV1 {
        message_id,
        contract_kind: SCHEDULER_COMMAND_KIND,
        envelope_sha256: Sha256::digest(&envelope_bytes).into(),
        envelope_bytes,
    })
}

pub fn decode_scheduler_result_v1(
    exact_bytes: &[u8],
    context: &DelayedDeliverySchedulerResultContextV1,
) -> Result<DecodedDelayedDeliverySchedulerResultV1, DelayedDeliverySchedulerAdapterErrorV1> {
    validate_result_context(context)?;
    let envelope = DurableEnvelopeV1::decode(exact_bytes)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidEnvelope)?;
    validate_envelope_v1(&envelope)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidEnvelope)?;
    if envelope.contract.as_ref()
        != Some(&contract(
            context.contract_revision,
            context.contract_schema_sha256,
        ))
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::WrongContract);
    }
    if envelope
        .source
        .as_ref()
        .map(|source| source.module_id.as_str())
        != Some(SCHEDULER_RUNTIME_MODULE_ID_V1)
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::WrongSource);
    }
    let result_metadata = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) => result,
        _ => return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidResult),
    };
    if envelope.causation_message_id != context.expected_command_message_id
        || result_metadata.command_message_id != context.expected_command_message_id
        || result_metadata.execution_attempt == 0
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::WrongCorrelation);
    }
    let payload = SchedulerScheduleControlResultV1::decode(envelope.payload.as_slice())
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidResult)?;
    validate_scheduler_schedule_control_result_v1(&payload)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidResult)?;
    let delayed_operation_id = id16(&payload.schedule_id)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidResult)?;
    if envelope.correlation_id != payload.operation_id
        || result_metadata.command_id != payload.operation_id
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::WrongCorrelation);
    }
    let outcome = SchedulerScheduleControlOutcomeV1::try_from(payload.outcome)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidResult)?;
    let expected_outer_outcome = match outcome {
        SchedulerScheduleControlOutcomeV1::Ensured
        | SchedulerScheduleControlOutcomeV1::Cancelled
        | SchedulerScheduleControlOutcomeV1::TooLate => ResultOutcomeV1::Succeeded,
        SchedulerScheduleControlOutcomeV1::Rejected => ResultOutcomeV1::Rejected,
        SchedulerScheduleControlOutcomeV1::Unspecified => {
            return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidResult);
        }
    };
    if result_metadata.outcome != expected_outer_outcome as i32 {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidResult);
    }
    let result = match outcome {
        SchedulerScheduleControlOutcomeV1::Ensured => DecodedSchedulerScheduleResultV1::Ensured {
            schedule_revision: payload.schedule_revision,
        },
        SchedulerScheduleControlOutcomeV1::Cancelled => DecodedSchedulerScheduleResultV1::Cancelled,
        SchedulerScheduleControlOutcomeV1::TooLate => DecodedSchedulerScheduleResultV1::TooLate,
        SchedulerScheduleControlOutcomeV1::Rejected => DecodedSchedulerScheduleResultV1::Rejected {
            error_code: rejection_code(&payload.error_code)?,
        },
        SchedulerScheduleControlOutcomeV1::Unspecified => unreachable!(),
    };
    Ok(DecodedDelayedDeliverySchedulerResultV1 {
        delayed_operation_id,
        message_id: id16(&envelope.message_id)
            .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidResult)?,
        envelope_sha256: Sha256::digest(exact_bytes).into(),
        result,
        received_at_unix_millis: context.received_at_unix_millis,
    })
}

pub fn scheduler_result_causation_id_v1(
    exact_bytes: &[u8],
) -> Result<[u8; 16], DelayedDeliverySchedulerAdapterErrorV1> {
    let envelope = DurableEnvelopeV1::decode(exact_bytes)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidEnvelope)?;
    validate_envelope_v1(&envelope)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::InvalidEnvelope)?;
    let result = match envelope.semantics.as_ref() {
        Some(Semantics::Result(result)) => result,
        _ => return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidResult),
    };
    if envelope.causation_message_id != result.command_message_id {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::WrongCorrelation);
    }
    id16(&envelope.causation_message_id)
        .map_err(|_| DelayedDeliverySchedulerAdapterErrorV1::WrongCorrelation)
}

fn validate_command_context(
    context: &DelayedDeliverySchedulerCommandContextV1,
) -> Result<(), DelayedDeliverySchedulerAdapterErrorV1> {
    if context.logical_owner_id.trim().is_empty()
        || context.logical_owner_id.len() > 128
        || context.runtime_instance_id.iter().all(|byte| *byte == 0)
        || context.runtime_generation == 0
        || context.grant_epoch == 0
        || context.contract_revision == 0
        || context.contract_schema_sha256.iter().all(|byte| *byte == 0)
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
        || context.deadline_unix_seconds < context.recorded_at_unix_seconds
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_result_context(
    context: &DelayedDeliverySchedulerResultContextV1,
) -> Result<(), DelayedDeliverySchedulerAdapterErrorV1> {
    if context
        .expected_command_message_id
        .iter()
        .all(|byte| *byte == 0)
        || context.contract_revision == 0
        || context.contract_schema_sha256.iter().all(|byte| *byte == 0)
        || context.received_at_unix_millis == 0
    {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidContext);
    }
    Ok(())
}

fn timestamp(
    seconds: i64,
    nanos: i32,
) -> Result<Timestamp, DelayedDeliverySchedulerAdapterErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) {
        return Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidContext);
    }
    Ok(Timestamp { seconds, nanos })
}

fn contract(revision: u32, schema_sha256: [u8; 32]) -> ContractRefV1 {
    ContractRefV1 {
        owner: SCHEDULER_CONTRACT_OWNER.to_owned(),
        name: SCHEDULER_CONTRACT_NAME.to_owned(),
        major: 1,
        revision,
        schema_sha256: schema_sha256.to_vec(),
    }
}

fn derived_id(domain: &[u8], operation_id: &[u8; 16]) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(domain)
        .chain_update(operation_id)
        .finalize();
    digest[..16].try_into().expect("SHA-256 prefix is exact")
}

fn idempotency_key(logical_owner_id: &str, operation_id: &[u8; 16]) -> Vec<u8> {
    Sha256::new()
        .chain_update(IDEMPOTENCY_DOMAIN)
        .chain_update(logical_owner_id.as_bytes())
        .chain_update(operation_id)
        .finalize()
        .to_vec()
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let value: [u8; 16] = value.try_into().map_err(|_| ())?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(())
}

fn rejection_code(value: &str) -> Result<u16, DelayedDeliverySchedulerAdapterErrorV1> {
    match value {
        "foreign_authority" => Ok(1),
        "unknown_schedule" => Ok(2),
        "stale_revision" => Ok(3),
        "revision_conflict" => Ok(4),
        "concurrency_busy" => Ok(5),
        _ => Err(DelayedDeliverySchedulerAdapterErrorV1::InvalidResult),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::{ResultMetadataV1, durable_envelope_v1::Semantics};
    use makosh_scheduler_protocol::v1::{
        EnsureOneShotScheduleV1, JobKindV1, SchedulerScheduleControlCommandV1,
        SchedulerScheduleControlResultV1, scheduler_schedule_control_command_v1::Operation,
    };

    fn command() -> SchedulerScheduleControlCommandV1 {
        SchedulerScheduleControlCommandV1 {
            operation_id: vec![7; 16],
            operation: Some(Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
                schedule_id: vec![9; 16],
                schedule_revision: 1,
                job_kind: Some(JobKindV1 {
                    owner: "communication_delayed_delivery".to_owned(),
                    name: "execute".to_owned(),
                    major: 1,
                }),
                job_contract_revision: 1,
                job_schema_sha256: vec![3; 32],
                scope_id: "07070707070707070707070707070707".to_owned(),
                concurrency_key: "07070707070707070707070707070707".to_owned(),
                due_at_unix_millis: 2_000_000,
                deadline_millis: 30_000,
                max_attempts: 3,
                retry_base_backoff_millis: 1_000,
            })),
        }
    }

    fn command_context() -> DelayedDeliverySchedulerCommandContextV1 {
        DelayedDeliverySchedulerCommandContextV1 {
            logical_owner_id: "owner-1".to_owned(),
            runtime_instance_id: [4; 16],
            runtime_generation: 2,
            grant_epoch: 3,
            contract_revision: 1,
            contract_schema_sha256: [5; 32],
            recorded_at_unix_seconds: 1_000,
            recorded_at_nanos: 0,
            deadline_unix_seconds: 1_030,
        }
    }

    #[test]
    fn command_is_stable_and_uses_grant_epoch_fence() {
        let first = build_scheduler_command_v1(command(), &command_context()).expect("command");
        let second = build_scheduler_command_v1(command(), &command_context()).expect("command");
        assert_eq!(first, second);
        let envelope = DurableEnvelopeV1::decode(first.envelope_bytes.as_slice()).expect("decode");
        assert_eq!(
            envelope.source_fence.expect("fence").kind,
            FenceKindV1::GrantEpoch as i32
        );
        assert!(
            !first
                .envelope_bytes
                .windows(7)
                .any(|bytes| bytes == b"body-v1")
        );
    }

    #[test]
    fn result_requires_exact_causation_and_maps_rejection() {
        let command = build_scheduler_command_v1(command(), &command_context()).expect("command");
        let payload = SchedulerScheduleControlResultV1 {
            operation_id: vec![7; 16],
            schedule_id: vec![9; 16],
            schedule_revision: 1,
            outcome: SchedulerScheduleControlOutcomeV1::Rejected as i32,
            error_code: "stale_revision".to_owned(),
        };
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: vec![8; 16],
            contract: Some(contract(1, [5; 32])),
            source: Some(SourceRefV1 {
                module_id: "makosh-scheduler-runtime".to_owned(),
                runtime_instance_id: vec![6; 16],
                runtime_generation: 2,
            }),
            recorded_at: Some(Timestamp {
                seconds: 1_001,
                nanos: 0,
            }),
            partition_key: vec![7; 16],
            causation_message_id: command.message_id.to_vec(),
            correlation_id: vec![7; 16],
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::System as i32,
                actor_id: b"makosh-scheduler-runtime".to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: b"makosh-scheduler-runtime".to_vec(),
                epoch: 2,
            }),
            semantics: Some(Semantics::Result(ResultMetadataV1 {
                command_id: vec![7; 16],
                command_message_id: command.message_id.to_vec(),
                outcome: ResultOutcomeV1::Rejected as i32,
                completed_at: Some(Timestamp {
                    seconds: 1_001,
                    nanos: 0,
                }),
                execution_attempt: 1,
            })),
            payload: payload.encode_to_vec(),
        };
        let bytes = envelope.encode_to_vec();
        assert_eq!(
            scheduler_result_causation_id_v1(&bytes).expect("causation"),
            command.message_id
        );
        let decoded = decode_scheduler_result_v1(
            &bytes,
            &DelayedDeliverySchedulerResultContextV1 {
                expected_command_message_id: command.message_id,
                contract_revision: 1,
                contract_schema_sha256: [5; 32],
                received_at_unix_millis: 1_001_000,
            },
        )
        .expect("result");
        assert_eq!(
            decoded.result,
            DecodedSchedulerScheduleResultV1::Rejected { error_code: 3 }
        );
        assert_eq!(
            decoded.delayed_operation_id, [9; 16],
            "workflow ownership follows Scheduler schedule_id, not command operation_id"
        );
    }
}
