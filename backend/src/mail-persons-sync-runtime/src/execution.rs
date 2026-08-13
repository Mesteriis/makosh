use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailPersonSourceContractV1, validate_mail_person_source_observed_v1,
    validate_mail_person_source_removed_v1, validate_mail_person_source_updated_v1,
    wire_person_source::{
        MailPersonSourceObservedV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_persons_sync_core::{
    map_observed_to_persons_v1, map_removed_to_persons_v1, map_updated_to_persons_v1,
};
use makosh_mail_persons_sync_persistence::{
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncPersistenceErrorV1,
    MailPersonsSyncPersistenceV1, MailPersonsSyncRunContextV1, StageMailPersonsSyncSourceV1,
};
use makosh_persons_api::wire::{PersonsCommandV1, persons_command_v1::Command};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};
use std::time::Duration;

use crate::{
    MailPersonSourceInputV1, MailPersonsSyncEnvelopeContextV1,
    build_persons_command_outbox_record_v1, dispatch_mail_person_source_v1,
};

const PERSONS_COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncExecutionErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    PageNotReady,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_mail_person_source_once_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    contract: MailPersonSourceContractV1,
    context: &MailPersonsSyncExecutionContextV1,
) -> Result<bool, MailPersonsSyncExecutionErrorV1> {
    if !matches!(
        contract,
        MailPersonSourceContractV1::SourceObserved
            | MailPersonSourceContractV1::SourceUpdated
            | MailPersonSourceContractV1::SourceRemoved
    ) {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload);
    }
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailPersonsSyncExecutionErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let run_id = source_run_id_v1(&record, contract)?;
    let run = persistence
        .load_run_context(&context.logical_owner_id, run_id)
        .await
        .map_err(MailPersonsSyncExecutionErrorV1::Persistence)?;
    let staged = match prepare_source_v1(&record, contract, &run, context) {
        Ok(staged) => staged,
        Err(MailPersonsSyncExecutionErrorV1::PageNotReady) => {
            delivery
                .retry_after(Duration::from_millis(100))
                .await
                .map_err(|_| MailPersonsSyncExecutionErrorV1::EventUnavailable)?;
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    persistence
        .stage_source_once(&staged)
        .await
        .map_err(MailPersonsSyncExecutionErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonsSyncExecutionErrorV1::EventUnavailable)?;
    Ok(true)
}

fn prepare_source_v1(
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
    run: &MailPersonsSyncRunContextV1,
    context: &MailPersonsSyncExecutionContextV1,
) -> Result<StageMailPersonsSyncSourceV1, MailPersonsSyncExecutionErrorV1> {
    validate_context(context)?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    validate_mail_envelope(&envelope, record, &contract.reference())?;
    let input = match contract {
        MailPersonSourceContractV1::SourceObserved => {
            let value = MailPersonSourceObservedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            validate_mail_person_source_observed_v1(&value)
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            MailPersonSourceInputV1::Observed(value)
        }
        MailPersonSourceContractV1::SourceUpdated => {
            let value = MailPersonSourceUpdatedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            validate_mail_person_source_updated_v1(&value)
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            MailPersonSourceInputV1::Updated(value)
        }
        MailPersonSourceContractV1::SourceRemoved => {
            let value = MailPersonSourceRemovedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            validate_mail_person_source_removed_v1(&value)
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
            MailPersonSourceInputV1::Removed(value)
        }
        _ => return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload),
    };
    let (observation_id, run_id, owner, page_sequence, source, revision, digest, observed_at) =
        source_identity(&input)?;
    let observed_at_unix_millis = timestamp_unix_millis_v1(&observed_at)
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    if owner != context.logical_owner_id
        || run.run_id != run_id
        || run.account_public_id != source.1
        || run.lease_expires_at_unix_millis <= context.now_unix_millis
        || run.lease_expires_at_unix_millis / 1_000 <= context.now_unix_millis / 1_000
        || observed_at_unix_millis > context.now_unix_millis
        || observed_at_unix_millis > run.lease_expires_at_unix_millis
        || observation_id.as_slice() != record.message_id()
        || envelope.partition_key != run_id
        || envelope.correlation_id != run_id
    {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload);
    }
    if matches!(run.state, 1 | 2) && page_sequence > run.next_page_sequence {
        return Err(MailPersonsSyncExecutionErrorV1::PageNotReady);
    }
    if !matches!(run.state, 1 | 2) || run.next_page_sequence != page_sequence {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload);
    }
    validate_observation_semantics_v1(
        &envelope,
        run_id,
        page_sequence,
        observation_id,
        digest,
        &observed_at,
    )?;
    let command = dispatch_mail_person_source_v1(input)
        .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    let command_id = persons_command_id(&command)?;
    let command_fingerprint: [u8; 32] = Sha256::digest(command.encode_to_vec()).into();
    let outbox = build_persons_command_outbox_record_v1(
        command,
        (context.now_unix_millis / 1_000 + PERSONS_COMMAND_DEADLINE_SECONDS_V1)
            .min(run.lease_expires_at_unix_millis / 1_000),
        &MailPersonsSyncEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_millis / 1_000,
            recorded_at_nanos: i32::try_from((context.now_unix_millis % 1_000) * 1_000_000)
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?,
        },
    )
    .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    Ok(StageMailPersonsSyncSourceV1 {
        logical_owner_id: owner,
        account_public_id: source.1,
        run_id,
        page_sequence,
        observation: MailPersonsSyncEnvelopeRecordV1 {
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            envelope_bytes: record.exact_bytes().to_vec(),
        },
        integration_public_id: source.0,
        provider_source_contact_public_id: source.2,
        change_kind: match contract {
            MailPersonSourceContractV1::SourceObserved => 1,
            MailPersonSourceContractV1::SourceUpdated => 2,
            MailPersonSourceContractV1::SourceRemoved => 3,
            _ => unreachable!(),
        },
        source_revision: revision,
        source_digest: digest,
        persons_command_id: command_id,
        persons_command_fingerprint: command_fingerprint,
        persons_command: MailPersonsSyncEnvelopeRecordV1 {
            message_id: *outbox.message_id(),
            envelope_sha256: *outbox.envelope_sha256(),
            envelope_bytes: outbox.exact_bytes().to_vec(),
        },
        received_at_unix_millis: context.now_unix_millis,
    })
}

fn source_run_id_v1(
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
) -> Result<[u8; 16], MailPersonsSyncExecutionErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let run_id = match contract {
        MailPersonSourceContractV1::SourceObserved => {
            MailPersonSourceObservedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?
                .run_id
        }
        MailPersonSourceContractV1::SourceUpdated => {
            MailPersonSourceUpdatedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?
                .run_id
        }
        MailPersonSourceContractV1::SourceRemoved => {
            MailPersonSourceRemovedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncExecutionErrorV1::InvalidPayload)?
                .run_id
        }
        _ => return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload),
    };
    id16(&run_id)
}

type SourceTuple = (
    [u8; 16],
    [u8; 16],
    String,
    u64,
    ([u8; 16], [u8; 16], [u8; 16]),
    u64,
    [u8; 32],
    prost_types::Timestamp,
);

fn source_identity(
    input: &MailPersonSourceInputV1,
) -> Result<SourceTuple, MailPersonsSyncExecutionErrorV1> {
    let (observation_id, run_id, owner, page, source, provenance) = match input {
        MailPersonSourceInputV1::Observed(value) => (
            &value.observation_id,
            &value.run_id,
            value.logical_owner_id.as_str(),
            value.page_sequence,
            value.source.as_ref(),
            value.provenance.as_ref(),
        ),
        MailPersonSourceInputV1::Updated(value) => (
            &value.observation_id,
            &value.run_id,
            value.logical_owner_id.as_str(),
            value.page_sequence,
            value.source.as_ref(),
            value.provenance.as_ref(),
        ),
        MailPersonSourceInputV1::Removed(value) => (
            &value.observation_id,
            &value.run_id,
            value.logical_owner_id.as_str(),
            value.page_sequence,
            value.source.as_ref(),
            value.provenance.as_ref(),
        ),
    };
    let source = source.ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    let provenance = provenance.ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    let observed_at = provenance
        .observed_at
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)?;
    Ok((
        id16(observation_id)?,
        id16(run_id)?,
        owner.to_owned(),
        page,
        (
            id16(&source.integration_public_id)?,
            id16(&source.account_public_id)?,
            id16(&source.provider_source_contact_public_id)?,
        ),
        provenance.source_revision,
        id32(&provenance.source_digest)?,
        observed_at,
    ))
}

fn validate_observation_semantics_v1(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    run_id: [u8; 16],
    page_sequence: u64,
    observation_id: [u8; 16],
    source_digest: [u8; 32],
    observed_at: &prost_types::Timestamp,
) -> Result<(), MailPersonsSyncExecutionErrorV1> {
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidEnvelope);
    };
    let expected_causation = deterministic_id16_v1(
        b"mail-persons-sync.fetch-page.v1",
        &run_id,
        &page_sequence.to_be_bytes(),
    );
    if metadata.observation_id != observation_id
        || metadata.source_cursor_sha256 != source_digest
        || metadata.source_sequence != Some(page_sequence)
        || metadata.observed_at.as_ref() != Some(observed_at)
        || metadata.occurred_at.as_ref() != Some(observed_at)
        || envelope.recorded_at.as_ref() != Some(observed_at)
        || envelope.causation_message_id != expected_causation
    {
        Err(MailPersonsSyncExecutionErrorV1::InvalidEnvelope)
    } else {
        Ok(())
    }
}

fn deterministic_id16_v1(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [label, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn timestamp_unix_millis_v1(value: &prost_types::Timestamp) -> Option<i64> {
    if value.seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return None;
    }
    value
        .seconds
        .checked_mul(1_000)?
        .checked_add(i64::from(value.nanos / 1_000_000))
}

fn persons_command_id(
    command: &PersonsCommandV1,
) -> Result<[u8; 16], MailPersonsSyncExecutionErrorV1> {
    let bytes = match command.command.as_ref() {
        Some(Command::SourceObserve(value)) => &value.command_id,
        Some(Command::SourceUpdate(value)) => &value.command_id,
        Some(Command::SourceRemove(value)) => &value.command_id,
        _ => return Err(MailPersonsSyncExecutionErrorV1::InvalidPayload),
    };
    id16(bytes)
}

fn validate_mail_envelope(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    record: &OutboxRecordV1,
    expected: &ContractReferenceV1,
) -> Result<(), MailPersonsSyncExecutionErrorV1> {
    crate::inbound::validate_exact_inbound_identity_v1(
        envelope,
        record,
        crate::inbound::ExactInboundIdentityV1 {
            contract: expected,
            source_module_id: MAIL_RUNTIME_MODULE_ID_V1,
            actor_kind: ActorKindV1::Module,
        },
    )
    .map_err(|()| MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidEnvelope);
    };
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || metadata.observation_id.as_slice() != record.message_id()
        || metadata.source_cursor_sha256.len() != 32
        || source.module_id != MAIL_RUNTIME_MODULE_ID_V1
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(MailPersonsSyncExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn validate_context(
    context: &MailPersonsSyncExecutionContextV1,
) -> Result<(), MailPersonsSyncExecutionErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.logical_owner_id.len() > 128
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
    {
        Err(MailPersonsSyncExecutionErrorV1::InvalidPayload)
    } else {
        Ok(())
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonsSyncExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], MailPersonsSyncExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 32]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncExecutionErrorV1::InvalidPayload)
}

#[allow(dead_code)]
fn _mapping_type_checks() {
    let _ = map_observed_to_persons_v1;
    let _ = map_updated_to_persons_v1;
    let _ = map_removed_to_persons_v1;
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::v1::durable_envelope_v1::Semantics;
    use makosh_mail_address_book_contract::{
        MailAddressBookEnvelopeContextV1, build_mail_person_source_observed_v1,
        mail_person_source_claims_digest_v1,
        wire_person_source::{
            MailPersonSourceClaimsV1, MailPersonSourceIdentityV1, MailPersonSourceObservedV1,
            MailPersonSourceProvenanceV1,
        },
    };
    use prost_types::Timestamp;

    use super::*;

    fn mutate_record(
        record: &OutboxRecordV1,
        mutate: impl FnOnce(&mut makosh_events_protocol::v1::DurableEnvelopeV1),
    ) -> OutboxRecordV1 {
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("decode fixture");
        mutate(&mut envelope);
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("accept mutated envelope")
    }

    fn fetch_id(run_id: [u8; 16], page_sequence: u64) -> [u8; 16] {
        let mut digest = Sha256::new();
        for part in [
            b"mail-persons-sync.fetch-page.v1".as_slice(),
            run_id.as_slice(),
            page_sequence.to_be_bytes().as_slice(),
        ] {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part);
        }
        digest.finalize()[..16].try_into().expect("digest prefix")
    }

    fn active_run() -> MailPersonsSyncRunContextV1 {
        MailPersonsSyncRunContextV1 {
            account_public_id: [0x31; 16],
            run_id: [0x51; 16],
            state: 1,
            next_page_sequence: 1,
            processed_pages: 0,
            processed_sources: 0,
            rejection_code: None,
            scheduler_message_id: [0x41; 16],
            lease_epoch: 1,
            lease_expires_at_unix_millis: 1_800_000_100_000,
        }
    }

    #[test]
    fn sanitized_mail_observation_prepares_one_exact_persons_command() {
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: vec![0x61; 16],
            account_public_id: vec![0x31; 16],
            provider_source_contact_public_id: vec![0x62; 16],
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: Some("Managed Public Person".to_owned()),
            normalized_emails: vec!["managed-public@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        let record = build_mail_person_source_observed_v1(
            fetch_id([0x51; 16], 1),
            MailPersonSourceObservedV1 {
                observation_id: vec![0x63; 16],
                run_id: vec![0x51; 16],
                logical_owner_id: "owner-1".to_owned(),
                page_sequence: 1,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: digest.to_vec(),
                    observed_at: Some(Timestamp {
                        seconds: 1_800_000_000,
                        nanos: 0,
                    }),
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-person-source-harness".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("sanitized observation");
        let staged = prepare_source_v1(
            &record,
            MailPersonSourceContractV1::SourceObserved,
            &active_run(),
            &MailPersonsSyncExecutionContextV1 {
                logical_owner_id: "owner-1".to_owned(),
                runtime_instance_id: "workflow-runtime".to_owned(),
                runtime_generation: 1,
                now_unix_millis: 1_800_000_000_000,
            },
        )
        .expect("prepare exact Persons command");
        assert_eq!(staged.account_public_id, [0x31; 16]);
        assert_eq!(staged.provider_source_contact_public_id, [0x62; 16]);
        assert_eq!(staged.change_kind, 1);
    }

    #[test]
    fn next_page_source_is_accepted_after_prior_page_continuation() {
        let mut run = active_run();
        run.state = 2;
        run.next_page_sequence = 2;
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: vec![0x61; 16],
            account_public_id: vec![0x31; 16],
            provider_source_contact_public_id: vec![0x62; 16],
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: None,
            normalized_emails: vec!["continued@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        let record = build_mail_person_source_observed_v1(
            fetch_id([0x51; 16], 2),
            MailPersonSourceObservedV1 {
                observation_id: vec![0x64; 16],
                run_id: vec![0x51; 16],
                logical_owner_id: "owner-1".to_owned(),
                page_sequence: 2,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: digest.to_vec(),
                    observed_at: Some(Timestamp {
                        seconds: 1_800_000_000,
                        nanos: 0,
                    }),
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-person-source-harness".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("continued observation");
        prepare_source_v1(
            &record,
            MailPersonSourceContractV1::SourceObserved,
            &run,
            &MailPersonsSyncExecutionContextV1 {
                logical_owner_id: "owner-1".to_owned(),
                runtime_instance_id: "workflow-runtime".to_owned(),
                runtime_generation: 1,
                now_unix_millis: 1_800_000_000_000,
            },
        )
        .expect("continued page source");
    }

    #[test]
    fn canonical_next_page_source_is_retryable_before_prior_completion() {
        let mut run = active_run();
        run.next_page_sequence = 1;
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: vec![0x61; 16],
            account_public_id: vec![0x31; 16],
            provider_source_contact_public_id: vec![0x62; 16],
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: None,
            normalized_emails: vec!["reordered@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        let record = build_mail_person_source_observed_v1(
            fetch_id([0x51; 16], 2),
            MailPersonSourceObservedV1 {
                observation_id: vec![0x65; 16],
                run_id: vec![0x51; 16],
                logical_owner_id: "owner-1".to_owned(),
                page_sequence: 2,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: digest.to_vec(),
                    observed_at: Some(Timestamp {
                        seconds: 1_800_000_000,
                        nanos: 0,
                    }),
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-person-source-harness".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("reordered source");
        assert_eq!(
            prepare_source_v1(
                &record,
                MailPersonSourceContractV1::SourceObserved,
                &run,
                &MailPersonsSyncExecutionContextV1 {
                    logical_owner_id: "owner-1".to_owned(),
                    runtime_instance_id: "workflow-runtime".to_owned(),
                    runtime_generation: 1,
                    now_unix_millis: 1_800_000_000_000,
                },
            ),
            Err(MailPersonsSyncExecutionErrorV1::PageNotReady),
        );
    }

    #[test]
    fn source_transition_is_rejected_after_run_lease_expiry() {
        let mut run = active_run();
        run.lease_expires_at_unix_millis = 1_800_000_000_000;
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: vec![0x61; 16],
            account_public_id: vec![0x31; 16],
            provider_source_contact_public_id: vec![0x62; 16],
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: None,
            normalized_emails: vec!["bounded@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        let record = build_mail_person_source_observed_v1(
            fetch_id([0x51; 16], 1),
            MailPersonSourceObservedV1 {
                observation_id: vec![0x63; 16],
                run_id: vec![0x51; 16],
                logical_owner_id: "owner-1".to_owned(),
                page_sequence: 1,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: digest.to_vec(),
                    observed_at: Some(Timestamp {
                        seconds: 1_800_000_001,
                        nanos: 0,
                    }),
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-person-source-harness".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("sanitized observation");
        assert_eq!(
            prepare_source_v1(
                &record,
                MailPersonSourceContractV1::SourceObserved,
                &run,
                &MailPersonsSyncExecutionContextV1 {
                    logical_owner_id: "owner-1".to_owned(),
                    runtime_instance_id: "workflow-runtime".to_owned(),
                    runtime_generation: 1,
                    now_unix_millis: 1_800_000_001_000,
                },
            ),
            Err(MailPersonsSyncExecutionErrorV1::InvalidPayload),
        );
    }

    #[test]
    fn observation_cursor_and_timestamps_are_exactly_bound() {
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: vec![0x61; 16],
            account_public_id: vec![0x31; 16],
            provider_source_contact_public_id: vec![0x62; 16],
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: None,
            normalized_emails: vec!["exact@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        let valid = build_mail_person_source_observed_v1(
            fetch_id([0x51; 16], 1),
            MailPersonSourceObservedV1 {
                observation_id: vec![0x63; 16],
                run_id: vec![0x51; 16],
                logical_owner_id: "owner-1".to_owned(),
                page_sequence: 1,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: digest.to_vec(),
                    observed_at: Some(Timestamp {
                        seconds: 1_800_000_000,
                        nanos: 0,
                    }),
                }),
            },
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-person-source-harness".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("sanitized observation");
        let context = MailPersonsSyncExecutionContextV1 {
            logical_owner_id: "owner-1".to_owned(),
            runtime_instance_id: "workflow-runtime".to_owned(),
            runtime_generation: 1,
            now_unix_millis: 1_800_000_000_000,
        };
        let mut run = active_run();
        run.lease_expires_at_unix_millis = 1_800_000_010_000;
        for invalid in [
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    panic!("observation metadata")
                };
                metadata.source_cursor_sha256 = vec![0x91; 32];
            }),
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    panic!("observation metadata")
                };
                metadata.source_sequence = Some(2);
            }),
            mutate_record(&valid, |envelope| {
                envelope.causation_message_id = vec![0x92; 16];
            }),
        ] {
            assert_eq!(
                prepare_source_v1(
                    &invalid,
                    MailPersonSourceContractV1::SourceObserved,
                    &run,
                    &context,
                ),
                Err(MailPersonsSyncExecutionErrorV1::InvalidEnvelope),
            );
        }
    }
}
