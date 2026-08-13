use makosh_calendar_api::{
    CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1, CalendarEnvelopeContextV1,
    build_calendar_event_changed_outbox_record_v1,
    client_wire::{CalendarEventChangedV1, CalendarEventStateV1 as WireEventState, TimestampV1},
};
use makosh_calendar_core::{CalendarEventRecordV1, CalendarEventStateV1, CalendarTimestampV1};
use makosh_calendar_persistence::{
    CalendarOutboxRecordV1, CalendarPersistenceErrorV1, CalendarPersistenceV1,
    CalendarSchedulerCommitV1, CalendarSchedulerInputV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePublishPermitV1, RuntimePullDeliveryV1,
    RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, ContractRefV1,
        DurableEnvelopeV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_scheduler_protocol::{
    SCHEDULER_RUNTIME_MODULE_ID_V1,
    v1::{
        JobLeaseV1, JobRunOutcomeV1, JobRunReceiptV1, JobTriggerKindV1, ScheduledJobCommandV1,
        SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1,
    },
    validate_job_run_receipt_v1, validate_scheduled_job_command_v1,
    validate_scheduler_schedule_control_result_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    admission::{
        scheduler_job_contract_v1, scheduler_receipt_contract_v1,
        scheduler_schedule_control_contract_v1,
    },
    scheduler::calendar_schedule_control_message_id_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarSchedulerRuntimeContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarSchedulerRuntimeErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(CalendarPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_calendar_schedule_result_once_v1(
    persistence: &CalendarPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<bool, CalendarSchedulerRuntimeErrorV1> {
    validate_runtime_context(context)?;
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let input = match decode_schedule_result(delivery.exact_bytes(), context) {
        Ok(input) => input,
        Err(_) => return acknowledge_invalid(delivery).await,
    };
    match persistence.record_scheduler_result_once(&input).await {
        Ok(_) | Err(CalendarPersistenceErrorV1::NotFound) => {}
        Err(error) => return Err(CalendarSchedulerRuntimeErrorV1::Persistence(error)),
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?;
    Ok(true)
}

pub async fn consume_calendar_reminder_due_once_v1(
    persistence: &CalendarPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<bool, CalendarSchedulerRuntimeErrorV1> {
    validate_runtime_context(context)?;
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let decoded = match decode_due(delivery.exact_bytes(), context) {
        Ok(decoded) => decoded,
        Err(_) => return acknowledge_invalid(delivery).await,
    };
    let result = persistence
        .apply_reminder_due_once(
            &decoded.input,
            timestamp(context.now_unix_millis)?,
            |event, changed| build_due_commit(event, changed, &decoded, context),
        )
        .await;
    match result {
        Ok(_) => {
            delivery
                .acknowledge()
                .await
                .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?;
            Ok(true)
        }
        Err(
            CalendarPersistenceErrorV1::NotFound | CalendarPersistenceErrorV1::OperationConflict,
        ) => {
            delivery
                .acknowledge()
                .await
                .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?;
            Ok(true)
        }
        Err(error) => Err(CalendarSchedulerRuntimeErrorV1::Persistence(error)),
    }
}

pub async fn relay_calendar_outbox_once_v1(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CalendarSchedulerRuntimeErrorV1> {
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(CalendarSchedulerRuntimeErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let hash = claim.record().envelope_sha256;
    connection
        .publish_exact(permit, &claim.record().envelope_bytes)
        .await
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?;
    claim
        .mark_published(hash, published_at_unix_millis)
        .await
        .map_err(CalendarSchedulerRuntimeErrorV1::Persistence)?;
    Ok(true)
}

struct DecodedDueV1 {
    input: CalendarSchedulerInputV1,
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    lease_epoch: u64,
    lease_expires_at_unix_millis: i64,
}

fn decode_due(
    bytes: &[u8],
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<DecodedDueV1, CalendarSchedulerRuntimeErrorV1> {
    let record = OutboxRecordV1::accept(bytes.to_vec())
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let envelope = exact_envelope(&record)?;
    validate_scheduler_source(&envelope, &record, &scheduler_job_contract_v1())?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope);
    };
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    if command.encode_to_vec() != envelope.payload {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    validate_scheduled_job_command_v1(&command)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let kind = command
        .job_kind
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let lease = command
        .lease
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let run_id = id16(&command.job_run_id)?;
    let reminder_id = id16(&command.schedule_id)?;
    let scheduled_at = timestamp_proto(command.scheduled_for_unix_millis)?;
    if kind.owner != CALENDAR_OWNER_ID_V1
        || kind.name != "reminder_due"
        || kind.major != 1
        || command.schedule_revision != 1
        || command.trigger_kind != JobTriggerKindV1::Time as i32
        || decode_id(&command.scope_id)? != reminder_id
        || envelope.partition_key != command.scope_id.as_bytes()
        || envelope.correlation_id != run_id
        || !envelope.causation_message_id.is_empty()
        || envelope.recorded_at.as_ref() != Some(&scheduled_at)
        || metadata.command_id != run_id
        || metadata.target_capability != "job_execute"
        || metadata.idempotency_key.len() != 32
        || metadata.idempotency_key.iter().all(|byte| *byte == 0)
        || metadata.logical_attempt != u32::try_from(lease.epoch).unwrap_or_default()
        || metadata.deadline.as_ref().and_then(timestamp_millis)
            != Some(lease.expires_at_unix_millis)
        || lease.run_id != run_id
        || lease.epoch == 0
    {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    Ok(DecodedDueV1 {
        input: CalendarSchedulerInputV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            envelope_bytes: record.exact_bytes().to_vec(),
            operation_kind: 2,
            reminder_id,
            expected_command_message_id: None,
            lease_expires_at_unix_millis: Some(lease.expires_at_unix_millis),
            completed_at_unix_millis: context.now_unix_millis,
        },
        command_message_id: *record.message_id(),
        run_id,
        lease_epoch: lease.epoch,
        lease_expires_at_unix_millis: lease.expires_at_unix_millis,
    })
}

fn decode_schedule_result(
    bytes: &[u8],
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<CalendarSchedulerInputV1, CalendarSchedulerRuntimeErrorV1> {
    let record = OutboxRecordV1::accept(bytes.to_vec())
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let envelope = exact_envelope(&record)?;
    validate_scheduler_source(
        &envelope,
        &record,
        &scheduler_schedule_control_contract_v1(),
    )?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope);
    };
    let result = SchedulerScheduleControlResultV1::decode(envelope.payload.as_slice())
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    if result.encode_to_vec() != envelope.payload {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    validate_scheduler_schedule_control_result_v1(&result)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let operation_id = id16(&result.operation_id)?;
    let reminder_id = id16(&result.schedule_id)?;
    let expected_command = calendar_schedule_control_message_id_v1(operation_id, reminder_id);
    let completed_at = metadata
        .completed_at
        .as_ref()
        .and_then(timestamp_millis)
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let expected_outcome = match SchedulerScheduleControlOutcomeV1::try_from(result.outcome)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?
    {
        SchedulerScheduleControlOutcomeV1::Rejected => ResultOutcomeV1::Rejected,
        SchedulerScheduleControlOutcomeV1::Ensured
        | SchedulerScheduleControlOutcomeV1::Cancelled
        | SchedulerScheduleControlOutcomeV1::TooLate => ResultOutcomeV1::Succeeded,
        SchedulerScheduleControlOutcomeV1::Unspecified => {
            return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
        }
    };
    if result.schedule_revision != 1
        || envelope.partition_key != owner_partition(&context.logical_owner_id)
        || envelope.correlation_id != reminder_id
        || envelope.causation_message_id != expected_command
        || envelope.recorded_at.as_ref().and_then(timestamp_millis) != Some(completed_at)
        || metadata.command_id != operation_id
        || metadata.command_message_id != expected_command
        || metadata.outcome != expected_outcome as i32
        || metadata.execution_attempt != 1
    {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    Ok(CalendarSchedulerInputV1 {
        logical_owner_id: context.logical_owner_id.clone(),
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
        operation_kind: 1,
        reminder_id,
        expected_command_message_id: Some(expected_command),
        lease_expires_at_unix_millis: None,
        completed_at_unix_millis: completed_at,
    })
}

fn build_due_commit(
    event: &CalendarEventRecordV1,
    changed: bool,
    due: &DecodedDueV1,
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<CalendarSchedulerCommitV1, CalendarPersistenceErrorV1> {
    let mut outbox = Vec::with_capacity(3);
    if changed {
        let lifecycle = build_calendar_event_changed_outbox_record_v1(
            due.run_id,
            CalendarEventChangedV1 {
                event_id: lifecycle_event_id(
                    due.run_id,
                    event.calendar_event_id,
                    event.event_revision,
                )
                .to_vec(),
                calendar_event_id: event.calendar_event_id.to_vec(),
                logical_owner_id: event.logical_owner_id.clone(),
                event_revision: event.event_revision,
                state: encode_event_state(event.state),
                occurred_at: Some(TimestampV1 {
                    unix_seconds: event.updated_at.unix_seconds,
                    nanos: event.updated_at.nanos,
                }),
            },
            &CalendarEnvelopeContextV1 {
                module_id: CALENDAR_MODULE_ID_V1.to_owned(),
                runtime_instance_id: context.runtime_instance_id.clone(),
                runtime_generation: context.runtime_generation,
                recorded_at_unix_seconds: context.now_unix_millis.div_euclid(1_000),
                recorded_at_nanos: nanos(context.now_unix_millis)
                    .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?,
            },
        )
        .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
        outbox.push(persistence_record(1, &lifecycle));
    }
    let accepted = build_receipt(due, JobRunOutcomeV1::Accepted, context)
        .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
    let succeeded = build_receipt(due, JobRunOutcomeV1::Succeeded, context)
        .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
    outbox.push(persistence_record(3, &accepted));
    outbox.push(persistence_record(4, &succeeded));
    Ok(CalendarSchedulerCommitV1 { outbox })
}

fn build_receipt(
    due: &DecodedDueV1,
    outcome: JobRunOutcomeV1,
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<OutboxRecordV1, CalendarSchedulerRuntimeErrorV1> {
    let payload = JobRunReceiptV1 {
        job_run_id: due.run_id.to_vec(),
        command_message_id: due.command_message_id.to_vec(),
        lease: Some(JobLeaseV1 {
            run_id: due.run_id.to_vec(),
            epoch: due.lease_epoch,
            expires_at_unix_millis: due.lease_expires_at_unix_millis,
        }),
        outcome: outcome as i32,
        observed_at_unix_millis: context.now_unix_millis,
    };
    validate_job_run_receipt_v1(&payload)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    let observed_at = timestamp_proto(context.now_unix_millis)?;
    let semantics = if outcome == JobRunOutcomeV1::Accepted {
        Semantics::Ack(AckMetadataV1 {
            acknowledged_message_id: due.command_message_id.to_vec(),
            stage: AckStageV1::DurableAcceptance as i32,
            disposition: AckDispositionV1::Applied as i32,
            acknowledged_at: Some(observed_at),
        })
    } else {
        Semantics::Result(ResultMetadataV1 {
            command_id: due.run_id.to_vec(),
            command_message_id: due.command_message_id.to_vec(),
            outcome: ResultOutcomeV1::Succeeded as i32,
            completed_at: Some(observed_at),
            execution_attempt: u32::try_from(due.lease_epoch)
                .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?,
        })
    };
    let contract = scheduler_receipt_contract_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: receipt_message_id(due.run_id, outcome).to_vec(),
        contract: Some(contract_ref(&contract)),
        source: Some(SourceRefV1 {
            module_id: CALENDAR_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_source_id(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(observed_at),
        partition_key: due.run_id.to_vec(),
        causation_message_id: due.command_message_id.to_vec(),
        correlation_id: due.run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: CALENDAR_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: CALENDAR_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(semantics),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn exact_envelope(
    record: &OutboxRecordV1,
) -> Result<DurableEnvelopeV1, CalendarSchedulerRuntimeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    if envelope.encode_to_vec() != record.exact_bytes() {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn validate_scheduler_source(
    envelope: &DurableEnvelopeV1,
    record: &OutboxRecordV1,
    expected: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> Result<(), CalendarSchedulerRuntimeErrorV1> {
    validate_envelope_v1(envelope).map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || source.module_id != SCHEDULER_RUNTIME_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::System as i32
        || actor.actor_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidEnvelope);
    }
    Ok(())
}

async fn acknowledge_invalid(
    delivery: RuntimePullDeliveryV1,
) -> Result<bool, CalendarSchedulerRuntimeErrorV1> {
    delivery
        .acknowledge()
        .await
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::EventUnavailable)?;
    Ok(true)
}

fn validate_runtime_context(
    context: &CalendarSchedulerRuntimeContextV1,
) -> Result<(), CalendarSchedulerRuntimeErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    Ok(())
}

fn persistence_record(semantic_kind: i16, record: &OutboxRecordV1) -> CalendarOutboxRecordV1 {
    CalendarOutboxRecordV1 {
        message_id: *record.message_id(),
        semantic_kind,
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn timestamp(value: i64) -> Result<CalendarTimestampV1, CalendarSchedulerRuntimeErrorV1> {
    Ok(CalendarTimestampV1 {
        unix_seconds: value.div_euclid(1_000),
        nanos: nanos(value)?,
    })
}

fn timestamp_proto(value: i64) -> Result<Timestamp, CalendarSchedulerRuntimeErrorV1> {
    Ok(Timestamp {
        seconds: value.div_euclid(1_000),
        nanos: nanos(value)?,
    })
}

fn timestamp_millis(value: &Timestamp) -> Option<i64> {
    if !(0..1_000_000_000).contains(&value.nanos) || value.nanos % 1_000_000 != 0 {
        return None;
    }
    value
        .seconds
        .checked_mul(1_000)?
        .checked_add(i64::from(value.nanos / 1_000_000))
}

fn nanos(value: i64) -> Result<i32, CalendarSchedulerRuntimeErrorV1> {
    i32::try_from(value.rem_euclid(1_000) * 1_000_000)
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], CalendarSchedulerRuntimeErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| CalendarSchedulerRuntimeErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(CalendarSchedulerRuntimeErrorV1::InvalidPayload)
}

fn decode_id(value: &str) -> Result<[u8; 16], CalendarSchedulerRuntimeErrorV1> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload);
    }
    let mut output = [0_u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex(pair[0])? << 4) | hex(pair[1])?;
    }
    id16(&output)
}

fn hex(value: u8) -> Result<u8, CalendarSchedulerRuntimeErrorV1> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(CalendarSchedulerRuntimeErrorV1::InvalidPayload),
    }
}

fn encode_event_state(value: CalendarEventStateV1) -> i32 {
    match value {
        CalendarEventStateV1::Scheduled => WireEventState::CalendarEventStateScheduled as i32,
        CalendarEventStateV1::Completed => WireEventState::CalendarEventStateCompleted as i32,
        CalendarEventStateV1::Cancelled => WireEventState::CalendarEventStateCancelled as i32,
    }
}

fn owner_partition(logical_owner_id: &str) -> [u8; 16] {
    digest16(
        b"makosh.calendar.owner-partition.v1\0",
        logical_owner_id.as_bytes(),
        b"calendar",
    )
}

fn runtime_source_id(runtime_instance_id: &str) -> [u8; 16] {
    digest16(
        b"calendar-runtime-instance-v1",
        runtime_instance_id.as_bytes(),
        b"source",
    )
}

fn receipt_message_id(run_id: [u8; 16], outcome: JobRunOutcomeV1) -> [u8; 16] {
    digest16(
        b"makosh.calendar.scheduler-receipt.v1\0",
        &run_id,
        &(outcome as i32).to_be_bytes(),
    )
}

fn lifecycle_event_id(
    operation_id: [u8; 16],
    calendar_event_id: [u8; 16],
    revision: u64,
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.calendar.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(calendar_event_id);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn digest16(domain: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((left.len() as u64).to_be_bytes());
    hash.update(left);
    hash.update((right.len() as u64).to_be_bytes());
    hash.update(right);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn contract_ref(value: &makosh_runtime_protocol::v1::ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: value.owner.clone(),
        name: value.name.clone(),
        major: value.major,
        revision: value.revision,
        schema_sha256: value.schema_sha256.clone(),
    }
}

fn outbox_error(_: OutboxRecordError) -> CalendarSchedulerRuntimeErrorV1 {
    CalendarSchedulerRuntimeErrorV1::InvalidEnvelope
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_message_and_owner_partition_are_exact_and_stable() {
        assert_eq!(
            calendar_schedule_control_message_id_v1([1; 16], [2; 16]),
            calendar_schedule_control_message_id_v1([1; 16], [2; 16])
        );
        assert_ne!(owner_partition("owner-1"), owner_partition("owner-2"));
        assert_eq!(decode_id(&"02".repeat(16)).expect("id"), [2; 16]);
    }
}
