use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, FenceKindV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_persons_sync_persistence::{
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncPersistenceV1, MailPersonsSyncSourceCommandContextV1,
    RecordMailPersonsSyncPersonsTerminalV1,
};
use makosh_persons_api::{
    PERSONS_MODULE_ID_V1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
    wire::{PersonCommandRejectedV1, PersonCommandSucceededV1},
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::page::{MailPersonsSyncPageContextV1, build_finished_page_outputs_v1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncPersonsTerminalContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncPersonsTerminalKindV1 {
    Succeeded,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncPersonsTerminalErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    EventUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalContextDispositionV1 {
    RecordKnown,
    AcknowledgeUnrelated,
}

fn classify_terminal_context_v1(
    context: Option<&MailPersonsSyncSourceCommandContextV1>,
) -> TerminalContextDispositionV1 {
    if context.is_some() {
        TerminalContextDispositionV1::RecordKnown
    } else {
        TerminalContextDispositionV1::AcknowledgeUnrelated
    }
}

pub async fn consume_mail_persons_sync_persons_terminal_once_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    kind: MailPersonsSyncPersonsTerminalKindV1,
    context: &MailPersonsSyncPersonsTerminalContextV1,
) -> Result<bool, MailPersonsSyncPersonsTerminalErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let validated = validate_terminal_v1(&record, kind, context)?;
    let command_id = validated.command_id;
    let source = persistence
        .find_source_command_context(&context.logical_owner_id, command_id)
        .await
        .map_err(MailPersonsSyncPersonsTerminalErrorV1::Persistence)?;
    if classify_terminal_context_v1(source.as_ref())
        == TerminalContextDispositionV1::AcknowledgeUnrelated
    {
        delivery
            .acknowledge()
            .await
            .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable)?;
        return Ok(true);
    }
    let source = source.expect("classified known source context");
    let recorded = persistence
        .record_persons_terminal_once(&RecordMailPersonsSyncPersonsTerminalV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            account_public_id: source.account_public_id,
            run_id: source.run_id,
            page_sequence: source.page_sequence,
            persons_command_id: command_id,
            result: MailPersonsSyncEnvelopeRecordV1 {
                message_id: *record.message_id(),
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            outcome: match kind {
                MailPersonsSyncPersonsTerminalKindV1::Succeeded => 1,
                MailPersonsSyncPersonsTerminalKindV1::Rejected => 2,
            },
            result_completed_at_unix_millis: validated.completed_at_unix_millis,
            received_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(MailPersonsSyncPersonsTerminalErrorV1::Persistence)?;
    if recorded.replayed {
        delivery
            .acknowledge()
            .await
            .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable)?;
        return Ok(true);
    }
    if let Some(finalization) = persistence
        .load_page_finalization_context(
            &context.logical_owner_id,
            source.run_id,
            source.page_sequence,
        )
        .await
        .map_err(MailPersonsSyncPersonsTerminalErrorV1::Persistence)?
    {
        let run = persistence
            .load_run_context(&context.logical_owner_id, source.run_id)
            .await
            .map_err(MailPersonsSyncPersonsTerminalErrorV1::Persistence)?;
        let (run_result, scheduler_terminal) = build_finished_page_outputs_v1(
            &run,
            &finalization,
            &MailPersonsSyncPageContextV1 {
                logical_owner_id: context.logical_owner_id.clone(),
                runtime_instance_id: context.runtime_instance_id.clone(),
                runtime_generation: context.runtime_generation,
                now_unix_millis: context.now_unix_millis,
            },
        )
        .map_err(|error| match error {
            crate::page::MailPersonsSyncPageErrorV1::Persistence(error) => {
                MailPersonsSyncPersonsTerminalErrorV1::Persistence(error)
            }
            crate::page::MailPersonsSyncPageErrorV1::EventUnavailable => {
                MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable
            }
            crate::page::MailPersonsSyncPageErrorV1::InvalidEnvelope => {
                MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope
            }
            crate::page::MailPersonsSyncPageErrorV1::InvalidPayload => {
                MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload
            }
        })?;
        persistence
            .finalize_finished_page_once(
                &context.logical_owner_id,
                source.run_id,
                source.page_sequence,
                run_result,
                scheduler_terminal,
                context.now_unix_millis,
            )
            .await
            .map_err(MailPersonsSyncPersonsTerminalErrorV1::Persistence)?;
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::EventUnavailable)?;
    Ok(true)
}

fn validate_terminal_v1(
    record: &OutboxRecordV1,
    kind: MailPersonsSyncPersonsTerminalKindV1,
    context: &MailPersonsSyncPersonsTerminalContextV1,
) -> Result<ValidatedPersonsTerminalV1, MailPersonsSyncPersonsTerminalErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let expected = match kind {
        MailPersonsSyncPersonsTerminalKindV1::Succeeded => {
            persons_command_succeeded_contract_reference_v1()
        }
        MailPersonsSyncPersonsTerminalKindV1::Rejected => {
            persons_command_rejected_contract_reference_v1()
        }
    };
    validate_envelope_identity(&envelope, record, &expected, kind)?;
    let command_id = match kind {
        MailPersonsSyncPersonsTerminalKindV1::Succeeded => {
            let payload = PersonCommandSucceededV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload)?;
            if payload.logical_owner_id != context.logical_owner_id
                || payload.resulting_owner_revision == 0
            {
                return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload);
            }
            id16(&payload.command_id)?
        }
        MailPersonsSyncPersonsTerminalKindV1::Rejected => {
            let payload = PersonCommandRejectedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload)?;
            if payload.logical_owner_id != context.logical_owner_id
                || payload.resulting_owner_revision == 0
            {
                return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload);
            }
            id16(&payload.command_id)?
        }
    };
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope);
    };
    let completed_at = metadata
        .completed_at
        .as_ref()
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let completed_at_unix_millis = timestamp_unix_millis_v1(completed_at)
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let owner_partition = deterministic_id16_v1(
        b"persons-owner-partition-v1",
        context.logical_owner_id.as_bytes(),
        b"persons",
    );
    let label = match kind {
        MailPersonsSyncPersonsTerminalKindV1::Succeeded => {
            b"persons-command-succeeded-v1".as_slice()
        }
        MailPersonsSyncPersonsTerminalKindV1::Rejected => b"persons-command-rejected-v1".as_slice(),
    };
    let expected_message_id =
        deterministic_id16_v1(label, &command_id, &Sha256::digest(&envelope.payload));
    if metadata.command_id != command_id
        || metadata.command_message_id != command_id
        || envelope.message_id != expected_message_id
        || envelope.partition_key != owner_partition
        || envelope.correlation_id != owner_partition
        || envelope.causation_message_id != command_id
        || envelope.recorded_at.as_ref() != Some(completed_at)
        || completed_at_unix_millis > context.now_unix_millis
    {
        return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope);
    }
    Ok(ValidatedPersonsTerminalV1 {
        command_id,
        completed_at_unix_millis,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ValidatedPersonsTerminalV1 {
    command_id: [u8; 16],
    completed_at_unix_millis: i64,
}

fn validate_envelope_identity(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    record: &OutboxRecordV1,
    expected: &ContractReferenceV1,
    kind: MailPersonsSyncPersonsTerminalKindV1,
) -> Result<(), MailPersonsSyncPersonsTerminalErrorV1> {
    crate::inbound::validate_exact_inbound_identity_v1(
        envelope,
        record,
        crate::inbound::ExactInboundIdentityV1 {
            contract: expected,
            source_module_id: PERSONS_MODULE_ID_V1,
            actor_kind: ActorKindV1::Module,
        },
    )
    .map_err(|()| MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let actual = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope);
    };
    let expected_outcome = match kind {
        MailPersonsSyncPersonsTerminalKindV1::Succeeded => ResultOutcomeV1::Succeeded,
        MailPersonsSyncPersonsTerminalKindV1::Rejected => ResultOutcomeV1::Rejected,
    };
    if actual.owner != expected.owner
        || actual.name != expected.name
        || actual.major != expected.major
        || actual.revision != expected.revision
        || actual.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || source.module_id != PERSONS_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != PERSONS_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || metadata.outcome != expected_outcome as i32
        || metadata.execution_attempt == 0
    {
        return Err(MailPersonsSyncPersonsTerminalErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn timestamp_unix_millis_v1(value: &prost_types::Timestamp) -> Option<i64> {
    if !(0..1_000_000_000).contains(&value.nanos) || value.nanos % 1_000_000 != 0 {
        return None;
    }
    value
        .seconds
        .checked_mul(1_000)?
        .checked_add(i64::from(value.nanos / 1_000_000))
}

fn deterministic_id16_v1(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update((first.len() as u64).to_be_bytes());
    digest.update(first);
    digest.update((second.len() as u64).to_be_bytes());
    digest.update(second);
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonsSyncPersonsTerminalErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncPersonsTerminalErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::v1::{
        ActorRefV1, ContractRefV1, DurableEnvelopeV1, ResultMetadataV1, SourceFenceV1, SourceRefV1,
    };
    use makosh_persons_api::wire::PersonRejectCodeV1;
    use prost_types::Timestamp;
    use sha2::{Digest, Sha256};

    use super::*;

    fn id16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
        let mut digest = Sha256::new();
        digest.update(label);
        digest.update((first.len() as u64).to_be_bytes());
        digest.update(first);
        digest.update((second.len() as u64).to_be_bytes());
        digest.update(second);
        digest.finalize()[..16].try_into().expect("digest prefix")
    }

    fn rejected_terminal() -> OutboxRecordV1 {
        let command_id = [0x41; 16];
        let owner = "owner-1";
        let payload = PersonCommandRejectedV1 {
            command_id: command_id.to_vec(),
            code: PersonRejectCodeV1::PersonRejectCodeConflict as i32,
            logical_owner_id: owner.to_owned(),
            resulting_owner_revision: 3,
        };
        let payload_bytes = payload.encode_to_vec();
        let partition = id16(b"persons-owner-partition-v1", owner.as_bytes(), b"persons");
        let message_id = id16(
            b"persons-command-rejected-v1",
            &command_id,
            &Sha256::digest(&payload_bytes),
        );
        let reference = persons_command_rejected_contract_reference_v1();
        let time = Timestamp {
            seconds: 1_800_000_000,
            nanos: 0,
        };
        OutboxRecordV1::accept(
            DurableEnvelopeV1 {
                envelope_major: 1,
                envelope_revision: 1,
                message_id: message_id.to_vec(),
                contract: Some(ContractRefV1 {
                    owner: reference.owner,
                    name: reference.name,
                    major: reference.major,
                    revision: reference.revision,
                    schema_sha256: reference.schema_sha256,
                }),
                source: Some(SourceRefV1 {
                    module_id: PERSONS_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: vec![0x42; 16],
                    runtime_generation: 1,
                }),
                recorded_at: Some(time),
                partition_key: partition.to_vec(),
                causation_message_id: command_id.to_vec(),
                correlation_id: partition.to_vec(),
                actor: Some(ActorRefV1 {
                    kind: ActorKindV1::Module as i32,
                    actor_id: PERSONS_MODULE_ID_V1.as_bytes().to_vec(),
                }),
                trace: None,
                source_fence: Some(SourceFenceV1 {
                    kind: FenceKindV1::RuntimeLease as i32,
                    scope_id: PERSONS_MODULE_ID_V1.as_bytes().to_vec(),
                    epoch: 1,
                }),
                semantics: Some(Semantics::Result(ResultMetadataV1 {
                    command_id: command_id.to_vec(),
                    command_message_id: command_id.to_vec(),
                    outcome: ResultOutcomeV1::Rejected as i32,
                    completed_at: Some(time),
                    execution_attempt: 1,
                })),
                payload: payload_bytes,
            }
            .encode_to_vec(),
        )
        .expect("terminal fixture")
    }

    fn mutate_terminal(
        record: &OutboxRecordV1,
        mutate: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> OutboxRecordV1 {
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("decode terminal");
        mutate(&mut envelope);
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("accept terminal mutation")
    }

    #[test]
    fn unrelated_exact_terminal_is_acknowledged_without_workflow_mutation() {
        assert_eq!(
            classify_terminal_context_v1(None),
            TerminalContextDispositionV1::AcknowledgeUnrelated,
        );
    }

    #[test]
    fn persons_terminal_exact_identity_outcome_time_and_causation_matrix() {
        let valid = rejected_terminal();
        let context = MailPersonsSyncPersonsTerminalContextV1 {
            logical_owner_id: "owner-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            now_unix_millis: 1_800_000_001_000,
        };
        validate_terminal_v1(
            &valid,
            MailPersonsSyncPersonsTerminalKindV1::Rejected,
            &context,
        )
        .expect("valid exact terminal");
        for (index, invalid) in [
            mutate_terminal(&valid, |envelope| envelope.partition_key = vec![9; 16]),
            mutate_terminal(&valid, |envelope| envelope.correlation_id = vec![9; 16]),
            mutate_terminal(&valid, |envelope| {
                envelope.causation_message_id = vec![9; 16]
            }),
            mutate_terminal(&valid, |envelope| {
                envelope.recorded_at = Some(Timestamp {
                    seconds: 1_800_000_001,
                    nanos: 0,
                });
            }),
            mutate_terminal(&valid, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result")
                };
                metadata.command_message_id = vec![9; 16];
            }),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                validate_terminal_v1(
                    &invalid,
                    MailPersonsSyncPersonsTerminalKindV1::Rejected,
                    &context,
                )
                .is_err(),
                "terminal mutation {index} was accepted",
            );
        }
    }
}
