use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, ContractRefV1,
        DurableEnvelopeV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_mail_address_book_contract::{
    MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1, MailAddressBookEnvelopeContextV1,
    build_fetch_mail_person_source_page_command_v1,
    wire_person_source::FetchMailPersonSourcePageCommandV1,
};
use makosh_mail_persons_sync_api::MAIL_PERSONS_SYNC_OWNER_V1;
use makosh_mail_persons_sync_persistence::{
    BeginMailPersonsSyncRunV1, MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncExpiredRunContextV1,
    MailPersonsSyncPersistenceErrorV1, MailPersonsSyncPersistenceV1,
    RejectMailPersonsSyncAccountBusyV1,
};
use makosh_scheduler_protocol::{
    SCHEDULER_RUNTIME_MODULE_ID_V1,
    v1::{JobLeaseV1, JobRunOutcomeV1, JobRunReceiptV1, JobTriggerKindV1, ScheduledJobCommandV1},
    validate_job_run_receipt_v1, validate_scheduled_job_command_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::admission::{scheduler_job_contract_v1, scheduler_receipt_contract_v1};
use crate::{
    MAIL_PERSONS_SYNC_MODULE_ID_V1, MailPersonsSyncEnvelopeContextV1, source_runtime_public_id_v1,
};

const JOB_NAME_V1: &str = "scheduled_sync";
const JOB_EXECUTE_CAPABILITY_V1: &str = "job_execute";
const COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncSchedulerContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncSchedulerErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_mail_persons_sync_due_once_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailPersonsSyncSchedulerContextV1,
) -> Result<bool, MailPersonsSyncSchedulerErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let begin = prepare_scheduler_due_v1(&record, context)?;
    match persistence
        .begin_run_reclaiming_expired_once(&begin, |expired: MailPersonsSyncExpiredRunContextV1| {
            let envelope_context = MailPersonsSyncEnvelopeContextV1 {
                runtime_instance_id: context.runtime_instance_id.clone(),
                runtime_generation: context.runtime_generation,
                recorded_at_unix_seconds: context.now_unix_millis / 1_000,
                recorded_at_nanos: nanos(context.now_unix_millis)
                    .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?,
            };
            let terminal = build_scheduler_receipt_v1(
                expired.scheduler_message_id,
                expired.run_id,
                expired.lease_epoch,
                expired.lease_expires_at_unix_millis,
                JobRunOutcomeV1::RetryableFailed,
                context,
                &envelope_context,
            )
            .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
            Ok(persistence_record(&terminal))
        })
        .await
    {
        Ok(_) => {}
        Err(MailPersonsSyncPersistenceErrorV1::AccountBusy) => {
            let rejection = build_account_busy_rejection_v1(&begin, context)?;
            persistence
                .record_account_busy_once(&rejection)
                .await
                .map_err(MailPersonsSyncSchedulerErrorV1::Persistence)?;
        }
        Err(error) => return Err(MailPersonsSyncSchedulerErrorV1::Persistence(error)),
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::EventUnavailable)?;
    Ok(true)
}

fn build_account_busy_rejection_v1(
    begin: &BeginMailPersonsSyncRunV1,
    context: &MailPersonsSyncSchedulerContextV1,
) -> Result<RejectMailPersonsSyncAccountBusyV1, MailPersonsSyncSchedulerErrorV1> {
    let envelope_context = MailPersonsSyncEnvelopeContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: nanos(context.now_unix_millis)?,
    };
    let terminal = build_scheduler_receipt_v1(
        begin.scheduler_command.message_id,
        begin.run_id,
        begin.lease_epoch,
        begin.lease_expires_at_unix_millis,
        JobRunOutcomeV1::RetryableFailed,
        context,
        &envelope_context,
    )?;
    Ok(RejectMailPersonsSyncAccountBusyV1 {
        begin: begin.clone(),
        scheduler_terminal: persistence_record(&terminal),
    })
}

fn prepare_scheduler_due_v1(
    record: &OutboxRecordV1,
    context: &MailPersonsSyncSchedulerContextV1,
) -> Result<BeginMailPersonsSyncRunV1, MailPersonsSyncSchedulerErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let expected = scheduler_job_contract_v1();
    crate::inbound::validate_exact_inbound_identity_v1(
        &envelope,
        record,
        crate::inbound::ExactInboundIdentityV1 {
            contract: &expected,
            source_module_id: SCHEDULER_RUNTIME_MODULE_ID_V1,
            actor_kind: ActorKindV1::System,
        },
    )
    .map_err(|()| MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let actual = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope);
    };
    if actual.owner != expected.owner
        || actual.name != expected.name
        || actual.major != expected.major
        || actual.revision != expected.revision
        || actual.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || !envelope.causation_message_id.is_empty()
        || source.module_id != SCHEDULER_RUNTIME_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::System as i32
        || actor.actor_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != SCHEDULER_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || metadata.target_capability != JOB_EXECUTE_CAPABILITY_V1
        || metadata.idempotency_key.len() != 32
        || metadata.idempotency_key.iter().all(|byte| *byte == 0)
    {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidEnvelope);
    }
    let command = ScheduledJobCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    validate_scheduled_job_command_v1(&command)
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    let kind = command
        .job_kind
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    let lease = command
        .lease
        .as_ref()
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    let run_id = id16(&command.job_run_id)?;
    let account_public_id = decode_account_scope_v1(&command.scope_id)?;
    if kind.owner != MAIL_PERSONS_SYNC_OWNER_V1
        || kind.name != JOB_NAME_V1
        || kind.major != 1
        || command.trigger_kind != JobTriggerKindV1::Time as i32
        || envelope.partition_key != command.scope_id.as_bytes()
        || envelope.correlation_id != run_id
        || envelope.recorded_at.as_ref()
            != Some(&Timestamp {
                seconds: command.scheduled_for_unix_millis.div_euclid(1_000),
                nanos: i32::try_from(
                    command.scheduled_for_unix_millis.rem_euclid(1_000) * 1_000_000,
                )
                .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?,
            })
        || metadata.command_id != run_id
        || lease.run_id != run_id
        || metadata.logical_attempt != u32::try_from(lease.epoch).unwrap_or_default()
        || lease.epoch == 0
        || !scheduler_deadline_matches_lease_v1(metadata.deadline.as_ref(), lease)
        || lease.expires_at_unix_millis <= context.now_unix_millis
        || lease.expires_at_unix_millis / 1_000 <= context.now_unix_millis / 1_000
    {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload);
    }
    let scheduler_message_id = *record.message_id();
    let envelope_context = MailPersonsSyncEnvelopeContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: nanos(context.now_unix_millis)?,
    };
    let acceptance = build_scheduler_receipt_v1(
        scheduler_message_id,
        run_id,
        lease.epoch,
        lease.expires_at_unix_millis,
        JobRunOutcomeV1::Accepted,
        context,
        &envelope_context,
    )?;
    let fetch_id = digest16(
        b"mail-persons-sync.fetch-page.v1",
        &run_id,
        &1_u64.to_be_bytes(),
    );
    let fetch = build_fetch_mail_person_source_page_command_v1(
        FetchMailPersonSourcePageCommandV1 {
            command_id: fetch_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: context.logical_owner_id.clone(),
            account_public_id: account_public_id.to_vec(),
            page_sequence: 1,
            page_size: MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1,
        },
        (context.now_unix_millis / 1_000 + COMMAND_DEADLINE_SECONDS_V1)
            .min(lease.expires_at_unix_millis / 1_000),
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_millis / 1_000,
            recorded_at_nanos: nanos(context.now_unix_millis)?,
        },
    )
    .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    Ok(BeginMailPersonsSyncRunV1 {
        logical_owner_id: context.logical_owner_id.clone(),
        account_public_id,
        run_id,
        run_fingerprint: Sha256::digest(record.exact_bytes()).into(),
        scheduler_command: persistence_record(record),
        scheduler_acceptance: persistence_record(&acceptance),
        initial_fetch: persistence_record(&fetch),
        lease_epoch: lease.epoch,
        lease_expires_at_unix_millis: lease.expires_at_unix_millis,
        received_at_unix_millis: context.now_unix_millis,
    })
}

fn scheduler_deadline_matches_lease_v1(deadline: Option<&Timestamp>, lease: &JobLeaseV1) -> bool {
    deadline.and_then(timestamp_unix_millis_v1) == Some(lease.expires_at_unix_millis)
}

fn timestamp_unix_millis_v1(value: &Timestamp) -> Option<i64> {
    if !(0..1_000_000_000).contains(&value.nanos) || value.nanos % 1_000_000 != 0 {
        return None;
    }
    value
        .seconds
        .checked_mul(1_000)?
        .checked_add(i64::from(value.nanos / 1_000_000))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_scheduler_receipt_v1(
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    lease_epoch: u64,
    lease_expires_at_unix_millis: i64,
    outcome: JobRunOutcomeV1,
    context: &MailPersonsSyncSchedulerContextV1,
    envelope_context: &MailPersonsSyncEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailPersonsSyncSchedulerErrorV1> {
    let payload = JobRunReceiptV1 {
        job_run_id: run_id.to_vec(),
        command_message_id: command_message_id.to_vec(),
        lease: Some(makosh_scheduler_protocol::v1::JobLeaseV1 {
            run_id: run_id.to_vec(),
            epoch: lease_epoch,
            expires_at_unix_millis: lease_expires_at_unix_millis,
        }),
        outcome: outcome as i32,
        observed_at_unix_millis: context.now_unix_millis,
    };
    validate_job_run_receipt_v1(&payload)
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?;
    let message_id = digest16(
        b"mail-persons-sync.scheduler-receipt.v1",
        &run_id,
        &(outcome as i32).to_be_bytes(),
    );
    let observed_at = timestamp(context.now_unix_millis)?;
    let semantics = match outcome {
        JobRunOutcomeV1::Accepted => Semantics::Ack(AckMetadataV1 {
            acknowledged_message_id: command_message_id.to_vec(),
            stage: AckStageV1::DurableAcceptance as i32,
            disposition: AckDispositionV1::Applied as i32,
            acknowledged_at: Some(observed_at),
        }),
        JobRunOutcomeV1::Succeeded
        | JobRunOutcomeV1::RetryableFailed
        | JobRunOutcomeV1::Failed
        | JobRunOutcomeV1::Cancelled => {
            let result_outcome = match outcome {
                JobRunOutcomeV1::Succeeded => ResultOutcomeV1::Succeeded,
                JobRunOutcomeV1::RetryableFailed | JobRunOutcomeV1::Failed => {
                    ResultOutcomeV1::Failed
                }
                JobRunOutcomeV1::Cancelled => ResultOutcomeV1::Cancelled,
                JobRunOutcomeV1::Accepted | JobRunOutcomeV1::Unspecified => unreachable!(),
            };
            Semantics::Result(ResultMetadataV1 {
                command_id: run_id.to_vec(),
                command_message_id: command_message_id.to_vec(),
                outcome: result_outcome as i32,
                completed_at: Some(observed_at),
                execution_attempt: u32::try_from(lease_epoch)
                    .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)?,
            })
        }
        JobRunOutcomeV1::Unspecified => {
            return Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload);
        }
    };
    let reference = scheduler_receipt_contract_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(contract_ref(reference)),
        source: Some(SourceRefV1 {
            module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: source_runtime_public_id_v1(envelope_context).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(observed_at),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(semantics),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec())
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidEnvelope)
}

pub fn decode_account_scope_v1(value: &str) -> Result<[u8; 16], MailPersonsSyncSchedulerErrorV1> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload);
    }
    let mut decoded = [0_u8; 16];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        decoded[index] = (hex(chunk[0])? << 4) | hex(chunk[1])?;
    }
    if decoded.iter().all(|byte| *byte == 0) || encode_account_scope_v1(decoded) != value {
        return Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload);
    }
    Ok(decoded)
}

#[must_use]
pub fn encode_account_scope_v1(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(32);
    for byte in value {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn hex(value: u8) -> Result<u8, MailPersonsSyncSchedulerErrorV1> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload),
    }
}

fn persistence_record(record: &OutboxRecordV1) -> MailPersonsSyncEnvelopeRecordV1 {
    MailPersonsSyncEnvelopeRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn contract_ref(value: makosh_runtime_protocol::v1::ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: value.owner,
        name: value.name,
        major: value.major,
        revision: value.revision,
        schema_sha256: value.schema_sha256,
    }
}

fn timestamp(value: i64) -> Result<Timestamp, MailPersonsSyncSchedulerErrorV1> {
    Ok(Timestamp {
        seconds: value / 1_000,
        nanos: nanos(value)?,
    })
}

fn nanos(value: i64) -> Result<i32, MailPersonsSyncSchedulerErrorV1> {
    i32::try_from((value % 1_000) * 1_000_000)
        .map_err(|_| MailPersonsSyncSchedulerErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonsSyncSchedulerErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncSchedulerErrorV1::InvalidPayload)
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [label, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

#[cfg(test)]
mod tests {
    use makosh_scheduler_protocol::v1::JobKindV1;

    use super::*;

    fn scheduler_due_record() -> OutboxRecordV1 {
        let now = 10_000_i64;
        let run_id = [2; 16];
        let account = [1; 16];
        let scope = encode_account_scope_v1(account);
        let reference = scheduler_job_contract_v1();
        let payload = ScheduledJobCommandV1 {
            job_run_id: run_id.to_vec(),
            job_kind: Some(JobKindV1 {
                owner: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
                name: JOB_NAME_V1.to_owned(),
                major: 1,
            }),
            schedule_id: vec![3; 16],
            schedule_revision: 1,
            scope_id: scope.clone(),
            trigger_kind: JobTriggerKindV1::Time as i32,
            scheduled_for_unix_millis: now,
            lease: Some(JobLeaseV1 {
                run_id: run_id.to_vec(),
                epoch: 1,
                expires_at_unix_millis: 20_000,
            }),
        };
        let envelope = DurableEnvelopeV1 {
            envelope_major: 1,
            envelope_revision: 1,
            message_id: vec![4; 16],
            contract: Some(contract_ref(reference)),
            source: Some(SourceRefV1 {
                module_id: SCHEDULER_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: vec![5; 16],
                runtime_generation: 1,
            }),
            recorded_at: Some(Timestamp {
                seconds: 10,
                nanos: 0,
            }),
            partition_key: scope.into_bytes(),
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
                epoch: 1,
            }),
            semantics: Some(Semantics::Command(
                makosh_events_protocol::v1::CommandMetadataV1 {
                    command_id: run_id.to_vec(),
                    target_capability: JOB_EXECUTE_CAPABILITY_V1.to_owned(),
                    idempotency_key: vec![6; 32],
                    deadline: Some(Timestamp {
                        seconds: 20,
                        nanos: 0,
                    }),
                    logical_attempt: 1,
                },
            )),
            payload: payload.encode_to_vec(),
        };
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("scheduler due fixture")
    }

    fn mutate_due(
        record: &OutboxRecordV1,
        mutate: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> OutboxRecordV1 {
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("decode due");
        mutate(&mut envelope);
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("accept due mutation")
    }

    #[test]
    fn scheduler_inbound_exact_identity_and_freshness_matrix() {
        let valid = scheduler_due_record();
        let context = MailPersonsSyncSchedulerContextV1 {
            logical_owner_id: "owner.a".to_owned(),
            runtime_instance_id: "runtime.a".to_owned(),
            runtime_generation: 1,
            now_unix_millis: 10_001,
        };
        prepare_scheduler_due_v1(&valid, &context).expect("valid exact due");
        for (index, invalid) in [
            mutate_due(&valid, |envelope| {
                envelope.causation_message_id = vec![7; 16]
            }),
            mutate_due(&valid, |envelope| envelope.correlation_id = vec![7; 16]),
            mutate_due(&valid, |envelope| {
                envelope.recorded_at = Some(Timestamp {
                    seconds: 11,
                    nanos: 0,
                });
            }),
            mutate_due(&valid, |envelope| {
                let Some(Semantics::Command(metadata)) = envelope.semantics.as_mut() else {
                    panic!("command")
                };
                metadata.idempotency_key = vec![0; 32];
            }),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                prepare_scheduler_due_v1(&invalid, &context).is_err(),
                "scheduler mutation {index} was accepted",
            );
        }
    }

    fn record(seed: u8) -> MailPersonsSyncEnvelopeRecordV1 {
        let envelope = build_fetch_mail_person_source_page_command_v1(
            FetchMailPersonSourcePageCommandV1 {
                command_id: vec![seed; 16],
                run_id: vec![250; 16],
                logical_owner_id: "owner.a".to_owned(),
                account_public_id: vec![251; 16],
                page_sequence: 1,
                page_size: 500,
            },
            2,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "scheduler-fixture".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1,
                recorded_at_nanos: 0,
            },
        )
        .expect("durable envelope");
        persistence_record(&envelope)
    }

    fn begin() -> BeginMailPersonsSyncRunV1 {
        BeginMailPersonsSyncRunV1 {
            logical_owner_id: "owner.a".to_owned(),
            account_public_id: [1; 16],
            run_id: [2; 16],
            run_fingerprint: [3; 32],
            scheduler_command: record(4),
            scheduler_acceptance: record(5),
            initial_fetch: record(6),
            lease_epoch: 7,
            lease_expires_at_unix_millis: 20_000,
            received_at_unix_millis: 10_000,
        }
    }

    #[test]
    fn account_scope_is_exact_lowercase_public_identity_only() {
        let id = [0xab; 16];
        let encoded = encode_account_scope_v1(id);
        assert_eq!(encoded, "abababababababababababababababab");
        assert_eq!(decode_account_scope_v1(&encoded), Ok(id));
        for invalid in [
            "ABABABABABABABABABABABABABABABAB",
            "abababababababababababababababa",
            "gbababababababababababababababab",
            "00000000000000000000000000000000",
            "private-account-id",
        ] {
            assert_eq!(
                decode_account_scope_v1(invalid),
                Err(MailPersonsSyncSchedulerErrorV1::InvalidPayload)
            );
        }
    }

    #[test]
    fn account_busy_is_a_durable_retryable_scheduler_terminal() {
        let context = MailPersonsSyncSchedulerContextV1 {
            logical_owner_id: "owner.a".to_owned(),
            runtime_instance_id: "runtime.a".to_owned(),
            runtime_generation: 9,
            now_unix_millis: 10_000,
        };
        let rejection = build_account_busy_rejection_v1(&begin(), &context)
            .expect("bounded account-busy rejection");
        let envelope = decode_envelope_v1(&rejection.scheduler_terminal.envelope_bytes)
            .expect("scheduler terminal envelope");
        let receipt = JobRunReceiptV1::decode(envelope.payload.as_slice())
            .expect("scheduler receipt payload");
        assert_eq!(receipt.outcome, JobRunOutcomeV1::RetryableFailed as i32);
        assert_eq!(receipt.job_run_id, vec![2; 16]);
        assert_eq!(receipt.command_message_id, vec![4; 16]);
        assert_eq!(rejection.begin, begin());
    }

    #[test]
    fn scheduler_receipt_semantics_match_acceptance_and_terminal_outcomes() {
        let context = MailPersonsSyncSchedulerContextV1 {
            logical_owner_id: "owner.a".to_owned(),
            runtime_instance_id: "runtime.a".to_owned(),
            runtime_generation: 9,
            now_unix_millis: 10_000,
        };
        let envelope_context = MailPersonsSyncEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: 10,
            recorded_at_nanos: 0,
        };
        let accepted = build_scheduler_receipt_v1(
            [4; 16],
            [2; 16],
            1,
            20_000,
            JobRunOutcomeV1::Accepted,
            &context,
            &envelope_context,
        )
        .expect("acceptance");
        let accepted = decode_envelope_v1(accepted.exact_bytes()).expect("acceptance envelope");
        assert!(matches!(accepted.semantics, Some(Semantics::Ack(_))));

        for (outcome, expected) in [
            (JobRunOutcomeV1::Succeeded, ResultOutcomeV1::Succeeded),
            (JobRunOutcomeV1::RetryableFailed, ResultOutcomeV1::Failed),
            (JobRunOutcomeV1::Failed, ResultOutcomeV1::Failed),
            (JobRunOutcomeV1::Cancelled, ResultOutcomeV1::Cancelled),
        ] {
            let record = build_scheduler_receipt_v1(
                [4; 16],
                [2; 16],
                1,
                20_000,
                outcome,
                &context,
                &envelope_context,
            )
            .expect("terminal");
            let envelope = decode_envelope_v1(record.exact_bytes()).expect("terminal envelope");
            let Some(Semantics::Result(result)) = envelope.semantics else {
                panic!("terminal receipt must use result semantics");
            };
            assert_eq!(result.outcome, expected as i32);
        }
    }

    #[test]
    fn scheduler_deadline_is_exactly_bound_to_the_lease() {
        let lease = JobLeaseV1 {
            run_id: vec![1; 16],
            epoch: 1,
            expires_at_unix_millis: 12_345,
        };
        assert!(scheduler_deadline_matches_lease_v1(
            Some(&Timestamp {
                seconds: 12,
                nanos: 345_000_000,
            }),
            &lease,
        ));
        assert!(!scheduler_deadline_matches_lease_v1(
            Some(&Timestamp {
                seconds: 12,
                nanos: 344_000_000,
            }),
            &lease,
        ));
    }
}
