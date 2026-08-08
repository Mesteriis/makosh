use makosh_communication_delayed_delivery_api::COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1;
use makosh_events_protocol::{
    envelope::validate_envelope_v1,
    v1::{
        AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, ContractRefV1,
        DurableEnvelopeV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_protocol::{
    SCHEDULER_RUNTIME_MODULE_ID_V1,
    v1::{JobLeaseV1, JobRunOutcomeV1, JobRunReceiptV1, JobTriggerKindV1, ScheduledJobCommandV1},
    validate_job_run_receipt_v1, validate_scheduled_job_command_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

const DELAYED_JOB_OWNER_V1: &str = "communication_delayed_delivery";
const DELAYED_JOB_NAME_V1: &str = "execute";
const DELAYED_JOB_MAJOR_V1: u32 = 1;
const JOB_EXECUTE_CAPABILITY_V1: &str = "job_execute";
const ACCEPTANCE_KIND_V1: &str = "scheduler.job_run.acceptance.v1";
const TERMINAL_KIND_V1: &str = "scheduler.job_run.result.v1";
const ACCEPTANCE_MESSAGE_DOMAIN_V1: &[u8] =
    b"makosh.communication-delayed-delivery.job-acceptance.v1\0";
const TERMINAL_MESSAGE_DOMAIN_V1: &[u8] =
    b"makosh.communication-delayed-delivery.job-terminal.v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDueContractV1 {
    pub job_revision: u32,
    pub job_schema_sha256: [u8; 32],
    pub receipt_revision: u32,
    pub receipt_schema_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDueRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub contract: DelayedDeliveryDueContractV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryDueMessageV1 {
    pub message_id: [u8; 16],
    pub contract_kind: &'static str,
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedDelayedDeliveryDueCommandV1 {
    pub delayed_operation_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub run_id: [u8; 16],
    pub schedule_revision: u64,
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: u64,
    pub acceptance_receipt: DelayedDeliveryDueMessageV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryDueAdapterErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongCommand,
    WrongCorrelation,
    InvalidTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReceiptBindingV1 {
    run_id: [u8; 16],
    command_message_id: [u8; 16],
    lease_epoch: u64,
    lease_expires_at_unix_millis: u64,
}

pub fn decode_delayed_delivery_due_command_v1(
    exact_bytes: &[u8],
    context: &DelayedDeliveryDueRuntimeContextV1,
) -> Result<DecodedDelayedDeliveryDueCommandV1, DelayedDeliveryDueAdapterErrorV1> {
    validate_context(context)?;
    let envelope = DurableEnvelopeV1::decode(exact_bytes)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidEnvelope)?;
    validate_envelope_v1(&envelope)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidEnvelope)?;
    let expected_contract = job_contract(&context.contract);
    if envelope.contract.as_ref() != Some(&expected_contract) {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongContract);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongSource)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongSource)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongSource)?;
    if source.module_id != SCHEDULER_RUNTIME_MODULE_ID_V1
        || actor.kind != ActorKindV1::System as i32
        || actor.actor_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongSource);
    }
    let metadata = match envelope.semantics.as_ref() {
        Some(Semantics::Command(metadata))
            if metadata.target_capability == JOB_EXECUTE_CAPABILITY_V1 =>
        {
            metadata
        }
        _ => return Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand),
    };
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    validate_scheduled_job_command_v1(&command)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    let job_kind = command
        .job_kind
        .as_ref()
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    if job_kind.owner != DELAYED_JOB_OWNER_V1
        || job_kind.name != DELAYED_JOB_NAME_V1
        || job_kind.major != DELAYED_JOB_MAJOR_V1
        || command.trigger_kind != JobTriggerKindV1::Time as i32
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand);
    }
    let delayed_operation_id =
        decode_scope_id(&command.scope_id).ok_or(DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    if command.schedule_id.as_slice() != delayed_operation_id
        || envelope.partition_key != command.scope_id.as_bytes()
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongCorrelation);
    }
    let run_id = id16(&command.job_run_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    if metadata.command_id != run_id
        || envelope.correlation_id != run_id
        || !envelope.causation_message_id.is_empty()
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongCorrelation);
    }
    let lease = command
        .lease
        .as_ref()
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    let lease_expires_at_unix_millis = positive_millis(lease.expires_at_unix_millis)?;
    let accepted_at_unix_millis = positive_millis(command.scheduled_for_unix_millis)?;
    let expected_attempt =
        u32::try_from(lease.epoch).map_err(|_| DelayedDeliveryDueAdapterErrorV1::WrongCommand)?;
    if lease.run_id.as_slice() != run_id
        || metadata.logical_attempt != expected_attempt
        || metadata.idempotency_key.len() != 32
        || metadata.idempotency_key.iter().all(|byte| *byte == 0)
        || timestamp_millis(
            metadata
                .deadline
                .as_ref()
                .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongCommand)?,
        )? != lease_expires_at_unix_millis
        || timestamp_millis(
            envelope
                .recorded_at
                .as_ref()
                .ok_or(DelayedDeliveryDueAdapterErrorV1::InvalidTime)?,
        )? != accepted_at_unix_millis
        || accepted_at_unix_millis >= lease_expires_at_unix_millis
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand);
    }
    let receipt_binding = ReceiptBindingV1 {
        run_id,
        command_message_id,
        lease_epoch: lease.epoch,
        lease_expires_at_unix_millis,
    };
    let acceptance_receipt = receipt_envelope(
        receipt_binding,
        accepted_at_unix_millis,
        JobRunOutcomeV1::Accepted,
        context,
    )?;
    Ok(DecodedDelayedDeliveryDueCommandV1 {
        delayed_operation_id,
        command_message_id,
        command_envelope_sha256: Sha256::digest(exact_bytes).into(),
        run_id,
        schedule_revision: command.schedule_revision,
        lease_epoch: lease.epoch,
        lease_expires_at_unix_millis,
        acceptance_receipt,
    })
}

pub fn build_delayed_delivery_terminal_receipt_v1(
    due: &DecodedDelayedDeliveryDueCommandV1,
    outcome: JobRunOutcomeV1,
    observed_at_unix_millis: u64,
    context: &DelayedDeliveryDueRuntimeContextV1,
) -> Result<DelayedDeliveryDueMessageV1, DelayedDeliveryDueAdapterErrorV1> {
    validate_context(context)?;
    if !matches!(
        outcome,
        JobRunOutcomeV1::Succeeded | JobRunOutcomeV1::Failed
    ) || observed_at_unix_millis == 0
        || observed_at_unix_millis >= due.lease_expires_at_unix_millis
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::InvalidTime);
    }
    receipt_envelope(
        ReceiptBindingV1 {
            run_id: due.run_id,
            command_message_id: due.command_message_id,
            lease_epoch: due.lease_epoch,
            lease_expires_at_unix_millis: due.lease_expires_at_unix_millis,
        },
        observed_at_unix_millis,
        outcome,
        context,
    )
}

fn receipt_envelope(
    binding: ReceiptBindingV1,
    observed_at_unix_millis: u64,
    outcome: JobRunOutcomeV1,
    context: &DelayedDeliveryDueRuntimeContextV1,
) -> Result<DelayedDeliveryDueMessageV1, DelayedDeliveryDueAdapterErrorV1> {
    let lease = JobLeaseV1 {
        run_id: binding.run_id.to_vec(),
        epoch: binding.lease_epoch,
        expires_at_unix_millis: i64::try_from(binding.lease_expires_at_unix_millis)
            .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?,
    };
    let observed_at = millis_timestamp(observed_at_unix_millis)?;
    let receipt = JobRunReceiptV1 {
        job_run_id: binding.run_id.to_vec(),
        command_message_id: binding.command_message_id.to_vec(),
        lease: Some(lease),
        outcome: outcome as i32,
        observed_at_unix_millis: i64::try_from(observed_at_unix_millis)
            .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?,
    };
    validate_job_run_receipt_v1(&receipt)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?;
    let (domain, kind, semantics) = match outcome {
        JobRunOutcomeV1::Accepted => (
            ACCEPTANCE_MESSAGE_DOMAIN_V1,
            ACCEPTANCE_KIND_V1,
            Semantics::Ack(AckMetadataV1 {
                acknowledged_message_id: binding.command_message_id.to_vec(),
                stage: AckStageV1::DurableAcceptance as i32,
                disposition: AckDispositionV1::Applied as i32,
                acknowledged_at: Some(observed_at),
            }),
        ),
        JobRunOutcomeV1::Succeeded | JobRunOutcomeV1::Failed => {
            let result_outcome = if outcome == JobRunOutcomeV1::Succeeded {
                ResultOutcomeV1::Succeeded
            } else {
                ResultOutcomeV1::Failed
            };
            (
                TERMINAL_MESSAGE_DOMAIN_V1,
                TERMINAL_KIND_V1,
                Semantics::Result(ResultMetadataV1 {
                    command_id: binding.run_id.to_vec(),
                    command_message_id: binding.command_message_id.to_vec(),
                    outcome: result_outcome as i32,
                    completed_at: Some(observed_at),
                    execution_attempt: u32::try_from(binding.lease_epoch)
                        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::WrongCommand)?,
                }),
            )
        }
        _ => return Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand),
    };
    let message_id = derived_message_id(domain, &binding.command_message_id);
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(receipt_contract(&context.contract)),
        source: Some(SourceRefV1 {
            module_id: COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(observed_at),
        partition_key: binding.run_id.to_vec(),
        causation_message_id: binding.command_message_id.to_vec(),
        correlation_id: binding.run_id.to_vec(),
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
        semantics: Some(semantics),
        payload: receipt.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidEnvelope)?;
    let envelope_bytes = envelope.encode_to_vec();
    Ok(DelayedDeliveryDueMessageV1 {
        message_id,
        contract_kind: kind,
        envelope_sha256: Sha256::digest(&envelope_bytes).into(),
        envelope_bytes,
    })
}

fn validate_context(
    context: &DelayedDeliveryDueRuntimeContextV1,
) -> Result<(), DelayedDeliveryDueAdapterErrorV1> {
    let contract = &context.contract;
    if context.runtime_instance_id.iter().all(|byte| *byte == 0)
        || context.runtime_generation == 0
        || context.grant_epoch == 0
        || contract.job_revision == 0
        || contract.job_schema_sha256.iter().all(|byte| *byte == 0)
        || contract.receipt_revision == 0
        || contract.receipt_schema_sha256.iter().all(|byte| *byte == 0)
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::InvalidContext);
    }
    Ok(())
}

fn job_contract(contract: &DelayedDeliveryDueContractV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: DELAYED_JOB_OWNER_V1.to_owned(),
        name: DELAYED_JOB_NAME_V1.to_owned(),
        major: DELAYED_JOB_MAJOR_V1,
        revision: contract.job_revision,
        schema_sha256: contract.job_schema_sha256.to_vec(),
    }
}

fn receipt_contract(contract: &DelayedDeliveryDueContractV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: "scheduler".to_owned(),
        name: "job_receipt".to_owned(),
        major: 1,
        revision: contract.receipt_revision,
        schema_sha256: contract.receipt_schema_sha256.to_vec(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], DelayedDeliveryDueAdapterErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::WrongCorrelation)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(DelayedDeliveryDueAdapterErrorV1::WrongCorrelation)
}

fn positive_millis(value: i64) -> Result<u64, DelayedDeliveryDueAdapterErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DelayedDeliveryDueAdapterErrorV1::InvalidTime)
}

fn millis_timestamp(value: u64) -> Result<Timestamp, DelayedDeliveryDueAdapterErrorV1> {
    let seconds =
        i64::try_from(value / 1_000).map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?;
    let nanos = i32::try_from((value % 1_000) * 1_000_000)
        .map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?;
    Ok(Timestamp { seconds, nanos })
}

fn timestamp_millis(value: &Timestamp) -> Result<u64, DelayedDeliveryDueAdapterErrorV1> {
    if value.seconds < 0
        || !(0..1_000_000_000).contains(&value.nanos)
        || value.nanos % 1_000_000 != 0
    {
        return Err(DelayedDeliveryDueAdapterErrorV1::InvalidTime);
    }
    let seconds =
        u64::try_from(value.seconds).map_err(|_| DelayedDeliveryDueAdapterErrorV1::InvalidTime)?;
    seconds
        .checked_mul(1_000)
        .and_then(|millis| millis.checked_add(u64::try_from(value.nanos / 1_000_000).ok()?))
        .ok_or(DelayedDeliveryDueAdapterErrorV1::InvalidTime)
}

fn decode_scope_id(value: &str) -> Option<[u8; 16]> {
    (value.len() == 32).then_some(())?;
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect::<Option<Vec<_>>>()?
        .try_into()
        .ok()
}

#[cfg(test)]
fn encode_scope_id(value: &[u8; 16]) -> String {
    use std::fmt::Write;
    value
        .iter()
        .fold(String::with_capacity(32), |mut out, byte| {
            write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
            out
        })
}

fn derived_message_id(domain: &[u8], command_message_id: &[u8; 16]) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(domain)
        .chain_update(command_message_id)
        .finalize();
    digest[..16].try_into().expect("SHA-256 prefix is exact")
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::CommandMetadataV1;
    use makosh_scheduler_protocol::v1::{JobKindV1, JobLeaseV1};

    fn context() -> DelayedDeliveryDueRuntimeContextV1 {
        DelayedDeliveryDueRuntimeContextV1 {
            runtime_instance_id: [6; 16],
            runtime_generation: 2,
            grant_epoch: 3,
            contract: DelayedDeliveryDueContractV1 {
                job_revision: 1,
                job_schema_sha256: [4; 32],
                receipt_revision: 1,
                receipt_schema_sha256: [5; 32],
            },
        }
    }

    fn due_envelope() -> Vec<u8> {
        let operation_id = [7; 16];
        let run_id = [8; 16];
        let command = ScheduledJobCommandV1 {
            job_run_id: run_id.to_vec(),
            job_kind: Some(JobKindV1 {
                owner: DELAYED_JOB_OWNER_V1.to_owned(),
                name: DELAYED_JOB_NAME_V1.to_owned(),
                major: 1,
            }),
            schedule_id: operation_id.to_vec(),
            schedule_revision: 1,
            scope_id: encode_scope_id(&operation_id),
            trigger_kind: JobTriggerKindV1::Time as i32,
            scheduled_for_unix_millis: 10_000,
            lease: Some(JobLeaseV1 {
                run_id: run_id.to_vec(),
                epoch: 1,
                expires_at_unix_millis: 20_000,
            }),
        };
        DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: vec![9; 16],
            contract: Some(job_contract(&context().contract)),
            source: Some(SourceRefV1 {
                module_id: SCHEDULER_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: vec![10; 16],
                runtime_generation: 4,
            }),
            recorded_at: Some(Timestamp {
                seconds: 10,
                nanos: 0,
            }),
            partition_key: encode_scope_id(&operation_id).into_bytes(),
            causation_message_id: Vec::new(),
            correlation_id: run_id.to_vec(),
            actor: Some(ActorRefV1 {
                kind: ActorKindV1::System as i32,
                actor_id: SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            }),
            trace: None,
            source_fence: Some(SourceFenceV1 {
                kind: FenceKindV1::RuntimeLease as i32,
                scope_id: SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
                epoch: 4,
            }),
            semantics: Some(Semantics::Command(CommandMetadataV1 {
                command_id: run_id.to_vec(),
                target_capability: JOB_EXECUTE_CAPABILITY_V1.to_owned(),
                idempotency_key: vec![11; 32],
                deadline: Some(Timestamp {
                    seconds: 20,
                    nanos: 0,
                }),
                logical_attempt: 1,
            })),
            payload: command.encode_to_vec(),
        }
        .encode_to_vec()
    }

    #[test]
    fn due_command_builds_stable_acceptance_and_terminal_receipts() {
        let due = decode_delayed_delivery_due_command_v1(&due_envelope(), &context()).expect("due");
        let duplicate =
            decode_delayed_delivery_due_command_v1(&due_envelope(), &context()).expect("duplicate");
        assert_eq!(due, duplicate);
        let terminal = build_delayed_delivery_terminal_receipt_v1(
            &due,
            JobRunOutcomeV1::Succeeded,
            15_000,
            &context(),
        )
        .expect("terminal");
        assert_eq!(terminal.contract_kind, TERMINAL_KIND_V1);
        assert_ne!(terminal.message_id, due.acceptance_receipt.message_id);
    }

    #[test]
    fn rejects_foreign_job_contract_before_scope_admission() {
        let mut envelope = DurableEnvelopeV1::decode(due_envelope().as_slice()).expect("envelope");
        envelope.contract.as_mut().expect("contract").owner = "mail".to_owned();
        assert_eq!(
            decode_delayed_delivery_due_command_v1(&envelope.encode_to_vec(), &context()),
            Err(DelayedDeliveryDueAdapterErrorV1::WrongContract)
        );
    }

    #[test]
    fn rejects_scheduler_metadata_that_does_not_bind_the_lease() {
        let mut envelope = DurableEnvelopeV1::decode(due_envelope().as_slice()).expect("envelope");
        let Some(Semantics::Command(metadata)) = envelope.semantics.as_mut() else {
            panic!("command semantics");
        };
        metadata.logical_attempt = 2;
        assert_eq!(
            decode_delayed_delivery_due_command_v1(&envelope.encode_to_vec(), &context()),
            Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand)
        );

        let mut envelope = DurableEnvelopeV1::decode(due_envelope().as_slice()).expect("envelope");
        let Some(Semantics::Command(metadata)) = envelope.semantics.as_mut() else {
            panic!("command semantics");
        };
        metadata.deadline = Some(Timestamp {
            seconds: 19,
            nanos: 0,
        });
        assert_eq!(
            decode_delayed_delivery_due_command_v1(&envelope.encode_to_vec(), &context()),
            Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand)
        );
    }

    #[test]
    fn rejects_recorded_time_that_does_not_bind_the_scheduled_time() {
        let mut envelope = DurableEnvelopeV1::decode(due_envelope().as_slice()).expect("envelope");
        envelope.recorded_at = Some(Timestamp {
            seconds: 11,
            nanos: 0,
        });
        assert_eq!(
            decode_delayed_delivery_due_command_v1(&envelope.encode_to_vec(), &context()),
            Err(DelayedDeliveryDueAdapterErrorV1::WrongCommand)
        );
    }
}
