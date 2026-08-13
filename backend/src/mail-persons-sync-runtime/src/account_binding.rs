use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailPersonSourceContractV1,
    validate_mail_person_source_account_ready_v1, validate_mail_person_source_account_retired_v1,
    wire_person_source::{MailPersonSourceAccountReadyV1, MailPersonSourceAccountRetiredV1},
};
use makosh_mail_persons_sync_api::MAIL_PERSONS_SYNC_OWNER_V1;
use makosh_mail_persons_sync_persistence::{
    ApplyMailPersonsSyncAccountLifecycleV1, MailPersonsSyncAccountLifecycleKindV1,
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncPersistenceV1,
};
use makosh_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1,
    v1::{
        CancelOneShotScheduleV1, EnsureOneShotScheduleV1, JobKindV1,
        SchedulerScheduleControlCommandV1, scheduler_schedule_control_command_v1::Operation,
    },
    validate_scheduler_schedule_control_command_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{MAIL_PERSONS_SYNC_MODULE_ID_V1, admission::scheduler_schedule_control_contract_v1};

const SCHEDULE_DELAY_MILLIS_V1: i64 = 1_000;
const SCHEDULE_DEADLINE_MILLIS_V1: u64 = 300_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncAccountBindingContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncAccountBindingErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_mail_person_source_account_lifecycle_once_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    contract: MailPersonSourceContractV1,
    context: &MailPersonsSyncAccountBindingContextV1,
) -> Result<bool, MailPersonsSyncAccountBindingErrorV1> {
    if !matches!(
        contract,
        MailPersonSourceContractV1::AccountReady | MailPersonSourceContractV1::AccountRetired
    ) {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidPayload);
    }
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    let input = decode_lifecycle(&record, contract, context)?;
    persistence
        .apply_account_lifecycle_once(&input, |schedule_revision| {
            build_schedule_control(&input, schedule_revision, context)
                .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)
        })
        .await
        .map_err(MailPersonsSyncAccountBindingErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::EventUnavailable)?;
    Ok(true)
}

fn decode_lifecycle(
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
    context: &MailPersonsSyncAccountBindingContextV1,
) -> Result<ApplyMailPersonsSyncAccountLifecycleV1, MailPersonsSyncAccountBindingErrorV1> {
    if context.logical_owner_id.is_empty()
        || runtime_source_reference(&context.runtime_instance_id).is_none()
        || context.runtime_generation == 0
        || context.grant_epoch == 0
        || context.now_unix_millis <= 0
    {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    if envelope.encode_to_vec() != record.exact_bytes() {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope);
    }
    let expected = contract.reference();
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    let actual = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    if actual.owner != expected.owner
        || actual.name != expected.name
        || actual.major != expected.major
        || actual.revision != expected.revision
        || actual.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || source.module_id != MAIL_RUNTIME_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope);
    }
    let (event_id, owner, integration, account, revision, occurred_at, kind, canonical_payload) =
        match contract {
            MailPersonSourceContractV1::AccountReady => {
                let value = MailPersonSourceAccountReadyV1::decode(envelope.payload.as_slice())
                    .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
                validate_mail_person_source_account_ready_v1(&value)
                    .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
                let canonical = value.encode_to_vec();
                (
                    value.account_event_id,
                    value.logical_owner_id,
                    value.integration_public_id,
                    value.account_public_id,
                    value.mapping_revision,
                    value.observed_at,
                    MailPersonsSyncAccountLifecycleKindV1::Ready,
                    canonical,
                )
            }
            MailPersonSourceContractV1::AccountRetired => {
                let value = MailPersonSourceAccountRetiredV1::decode(envelope.payload.as_slice())
                    .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
                validate_mail_person_source_account_retired_v1(&value)
                    .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
                let canonical = value.encode_to_vec();
                (
                    value.account_event_id,
                    value.logical_owner_id,
                    value.integration_public_id,
                    value.account_public_id,
                    value.mapping_revision,
                    value.retired_at,
                    MailPersonsSyncAccountLifecycleKindV1::Retired,
                    canonical,
                )
            }
            _ => return Err(MailPersonsSyncAccountBindingErrorV1::InvalidPayload),
        };
    let event_id = id16(&event_id)?;
    let integration = id16(&integration)?;
    let account = id16(&account)?;
    let occurred = occurred_at.ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
    let occurred_millis = occurred
        .seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(i64::from(occurred.nanos / 1_000_000)))
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
    let expected_event_id = lifecycle_id(
        match kind {
            MailPersonsSyncAccountLifecycleKindV1::Ready => {
                b"makosh.mail.person-source.account-ready.v1"
            }
            MailPersonsSyncAccountLifecycleKindV1::Retired => {
                b"makosh.mail.person-source.account-retired.v1"
            }
        },
        &owner,
        account,
        revision,
    );
    let expected_causation = lifecycle_id(
        match kind {
            MailPersonsSyncAccountLifecycleKindV1::Ready => {
                b"makosh.mail.person-source.account-ready-causation.v1"
            }
            MailPersonsSyncAccountLifecycleKindV1::Retired => {
                b"makosh.mail.person-source.account-retired-causation.v1"
            }
        },
        &owner,
        account,
        revision,
    );
    let expected_cursor = lifecycle_cursor(account, revision, contract);
    let Some(Semantics::Observation(ObservationMetadataV1 {
        observation_id,
        observed_at,
        occurred_at,
        source_cursor_sha256,
        source_sequence,
    })) = envelope.semantics.as_ref()
    else {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope);
    };
    if canonical_payload != envelope.payload
        || event_id != *record.message_id()
        || event_id != expected_event_id
        || owner != context.logical_owner_id
        || envelope.partition_key != account
        || envelope.correlation_id != account
        || envelope.causation_message_id != expected_causation
        || envelope.recorded_at.as_ref() != Some(&occurred)
        || observation_id != event_id.as_slice()
        || observed_at.as_ref() != Some(&occurred)
        || occurred_at.as_ref() != Some(&occurred)
        || source_cursor_sha256 != expected_cursor.as_slice()
        || *source_sequence != Some(revision)
        || occurred_millis > context.now_unix_millis
    {
        return Err(MailPersonsSyncAccountBindingErrorV1::InvalidPayload);
    }
    Ok(ApplyMailPersonsSyncAccountLifecycleV1 {
        logical_owner_id: owner,
        integration_public_id: integration,
        account_public_id: account,
        mapping_revision: revision,
        kind,
        lifecycle: MailPersonsSyncEnvelopeRecordV1 {
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            envelope_bytes: record.exact_bytes().to_vec(),
        },
        processed_at_unix_millis: context.now_unix_millis,
    })
}

fn build_schedule_control(
    input: &ApplyMailPersonsSyncAccountLifecycleV1,
    schedule_revision: u64,
    context: &MailPersonsSyncAccountBindingContextV1,
) -> Result<MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncAccountBindingErrorV1> {
    let schedule_id = derive16(
        b"mail-persons-sync-schedule-v1",
        &input.account_public_id,
        input.logical_owner_id.as_bytes(),
    );
    let operation_id = derive16(
        b"mail-persons-sync-schedule-operation-v1",
        &input.lifecycle.message_id,
        &schedule_revision.to_be_bytes(),
    );
    let job = JobKindV1 {
        owner: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
        name: "scheduled_sync".to_owned(),
        major: 1,
    };
    let operation = match input.kind {
        MailPersonsSyncAccountLifecycleKindV1::Ready => {
            Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
                schedule_id: schedule_id.to_vec(),
                schedule_revision,
                job_kind: Some(job),
                job_contract_revision: 1,
                job_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
                scope_id: hex(&input.account_public_id),
                concurrency_key: hex(&input.account_public_id),
                due_at_unix_millis: context
                    .now_unix_millis
                    .checked_add(SCHEDULE_DELAY_MILLIS_V1)
                    .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?,
                deadline_millis: SCHEDULE_DEADLINE_MILLIS_V1,
                max_attempts: 3,
                retry_base_backoff_millis: 1_000,
            })
        }
        MailPersonsSyncAccountLifecycleKindV1::Retired => {
            Operation::CancelOneShot(CancelOneShotScheduleV1 {
                schedule_id: schedule_id.to_vec(),
                expected_schedule_revision: schedule_revision
                    .checked_sub(1)
                    .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?,
                job_kind: Some(job),
            })
        }
    };
    let payload = SchedulerScheduleControlCommandV1 {
        operation_id: operation_id.to_vec(),
        operation: Some(operation),
    };
    validate_scheduler_schedule_control_command_v1(&payload)
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
    let contract = scheduler_schedule_control_contract_v1();
    let partition = derive16(
        b"mail-persons-sync-owner-partition-v1",
        input.logical_owner_id.as_bytes(),
        b"scheduler",
    );
    let seconds = context.now_unix_millis / 1_000;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: operation_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id)
                .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?
                .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds,
            nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
        }),
        partition_key: partition.to_vec(),
        causation_message_id: input.lifecycle.message_id.to_vec(),
        correlation_id: operation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::GrantEpoch as i32,
            scope_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.grant_epoch,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: "scheduler_schedule_control".to_owned(),
            idempotency_key: Sha256::digest(payload.encode_to_vec()).to_vec(),
            deadline: Some(Timestamp {
                seconds: seconds + 300,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)?;
    let bytes = envelope.encode_to_vec();
    Ok(MailPersonsSyncEnvelopeRecordV1 {
        message_id: operation_id,
        envelope_sha256: Sha256::digest(&bytes).into(),
        envelope_bytes: bytes,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonsSyncAccountBindingErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| MailPersonsSyncAccountBindingErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(MailPersonsSyncAccountBindingErrorV1::InvalidPayload)
}

fn runtime_source_reference(value: &str) -> Option<[u8; 16]> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    let mut result = [0_u8; 16];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).ok()?;
    }
    result.iter().any(|byte| *byte != 0).then_some(result)
}

fn derive16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(label);
    hash.update((first.len() as u64).to_be_bytes());
    hash.update(first);
    hash.update((second.len() as u64).to_be_bytes());
    hash.update(second);
    let digest = hash.finalize();
    let mut id = [0; 16];
    id.copy_from_slice(&digest[..16]);
    id
}

fn lifecycle_id(
    domain: &[u8],
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    mapping_revision: u64,
) -> [u8; 16] {
    let revision = mapping_revision.to_be_bytes();
    let mut digest = Sha256::new();
    for value in [
        domain,
        logical_owner_id.as_bytes(),
        account_public_id.as_slice(),
        revision.as_slice(),
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn lifecycle_cursor(
    account_public_id: [u8; 16],
    mapping_revision: u64,
    contract: MailPersonSourceContractV1,
) -> [u8; 32] {
    let mut cursor = Sha256::new();
    cursor.update(b"makosh.mail.person-source.account-lifecycle.v1");
    cursor.update(account_public_id);
    cursor.update(mapping_revision.to_be_bytes());
    cursor.update(contract.name().as_bytes());
    cursor.finalize().into()
}

fn hex(value: &[u8; 16]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_mail_address_book_contract::{
        MailAddressBookEnvelopeContextV1, build_mail_person_source_account_ready_v1,
        wire_person_source::MailPersonSourceAccountReadyV1,
    };
    #[test]
    fn schedule_identity_is_owner_account_scoped_and_private_free() {
        assert_ne!(
            derive16(b"mail-persons-sync-schedule-v1", &[1; 16], b"owner-a"),
            derive16(b"mail-persons-sync-schedule-v1", &[1; 16], b"owner-b")
        );
        assert_eq!(hex(&[0xab; 16]), "abababababababababababababababab");
    }

    #[test]
    fn schedule_control_uses_exact_inherited_runtime_authority() {
        let context = MailPersonsSyncAccountBindingContextV1 {
            logical_owner_id: "owner.a".to_owned(),
            runtime_instance_id: "11111111111111111111111111111111".to_owned(),
            runtime_generation: 7,
            grant_epoch: 11,
            now_unix_millis: 1_700_000_000_000,
        };
        let input = ApplyMailPersonsSyncAccountLifecycleV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            integration_public_id: [2; 16],
            account_public_id: [3; 16],
            mapping_revision: 1,
            kind: MailPersonsSyncAccountLifecycleKindV1::Ready,
            lifecycle: MailPersonsSyncEnvelopeRecordV1 {
                message_id: [4; 16],
                envelope_sha256: [5; 32],
                envelope_bytes: vec![6],
            },
            processed_at_unix_millis: context.now_unix_millis,
        };

        let record = build_schedule_control(&input, 1, &context).expect("schedule control");
        let envelope = decode_envelope_v1(&record.envelope_bytes).expect("envelope");
        let source = envelope.source.expect("source");
        let fence = envelope.source_fence.expect("fence");
        assert_eq!(source.runtime_instance_id, vec![0x11; 16]);
        assert_eq!(source.runtime_generation, context.runtime_generation);
        assert_eq!(fence.kind, FenceKindV1::GrantEpoch as i32);
        assert_eq!(fence.scope_id, MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes());
        assert_eq!(fence.epoch, context.grant_epoch);
    }

    #[test]
    fn retired_account_emits_exact_cancel_for_the_ready_schedule_revision() {
        let context = MailPersonsSyncAccountBindingContextV1 {
            logical_owner_id: "owner.a".to_owned(),
            runtime_instance_id: "11111111111111111111111111111111".to_owned(),
            runtime_generation: 7,
            grant_epoch: 11,
            now_unix_millis: 1_700_000_000_000,
        };
        let input = ApplyMailPersonsSyncAccountLifecycleV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            integration_public_id: [2; 16],
            account_public_id: [3; 16],
            mapping_revision: 1,
            kind: MailPersonsSyncAccountLifecycleKindV1::Retired,
            lifecycle: MailPersonsSyncEnvelopeRecordV1 {
                message_id: [4; 16],
                envelope_sha256: [5; 32],
                envelope_bytes: vec![6],
            },
            processed_at_unix_millis: context.now_unix_millis,
        };
        let record = build_schedule_control(&input, 2, &context).expect("cancel control");
        let envelope = decode_envelope_v1(&record.envelope_bytes).expect("envelope");
        let payload = SchedulerScheduleControlCommandV1::decode(envelope.payload.as_slice())
            .expect("schedule control");
        let Some(Operation::CancelOneShot(cancel)) = payload.operation else {
            panic!("cancel operation");
        };
        assert_eq!(cancel.expected_schedule_revision, 1);
    }

    #[test]
    fn lifecycle_requires_canonical_public_payload_and_exact_producer_semantics() {
        let owner = "owner-1";
        let account = [3; 16];
        let revision = 1;
        let event_id = lifecycle_id(
            b"makosh.mail.person-source.account-ready.v1",
            owner,
            account,
            revision,
        );
        let causation = lifecycle_id(
            b"makosh.mail.person-source.account-ready-causation.v1",
            owner,
            account,
            revision,
        );
        let record = build_mail_person_source_account_ready_v1(
            causation,
            MailPersonSourceAccountReadyV1 {
                account_event_id: event_id.to_vec(),
                logical_owner_id: owner.to_owned(),
                integration_public_id: vec![2; 16],
                account_public_id: account.to_vec(),
                mapping_revision: revision,
                observed_at: Some(Timestamp {
                    seconds: 10,
                    nanos: 0,
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 7,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("ready");
        let context = MailPersonsSyncAccountBindingContextV1 {
            logical_owner_id: owner.to_owned(),
            runtime_instance_id: "11111111111111111111111111111111".to_owned(),
            runtime_generation: 9,
            grant_epoch: 11,
            now_unix_millis: 10_000,
        };
        assert!(
            decode_lifecycle(&record, MailPersonSourceContractV1::AccountReady, &context).is_ok()
        );

        let mut raw_top_level_unknown = record.exact_bytes().to_vec();
        raw_top_level_unknown
            .extend_from_slice(&[0xa2, 0x06, 0x07, b'p', b'r', b'i', b'v', b'a', b't', b'e']);
        let raw_top_level_unknown = OutboxRecordV1::accept(raw_top_level_unknown)
            .expect("structurally valid unknown field");
        assert!(matches!(
            decode_lifecycle(
                &raw_top_level_unknown,
                MailPersonSourceContractV1::AccountReady,
                &context,
            ),
            Err(MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)
        ));

        for mutation in [
            |envelope: &mut DurableEnvelopeV1| envelope.causation_message_id = vec![9; 16],
            |envelope: &mut DurableEnvelopeV1| {
                envelope.recorded_at = Some(Timestamp {
                    seconds: 9,
                    nanos: 0,
                })
            },
            |envelope: &mut DurableEnvelopeV1| {
                envelope.payload.extend_from_slice(&[
                    0xa2, 0x06, 0x07, b'p', b'r', b'i', b'v', b'a', b't', b'e',
                ])
            },
            |envelope: &mut DurableEnvelopeV1| {
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    return;
                };
                metadata.source_cursor_sha256 = vec![8; 32];
            },
        ] {
            let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
            mutation(&mut envelope);
            let mutated =
                OutboxRecordV1::accept(envelope.encode_to_vec()).expect("structural record");
            assert!(matches!(
                decode_lifecycle(&mutated, MailPersonSourceContractV1::AccountReady, &context),
                Err(MailPersonsSyncAccountBindingErrorV1::InvalidPayload
                    | MailPersonsSyncAccountBindingErrorV1::InvalidEnvelope)
            ));
        }
    }
}
