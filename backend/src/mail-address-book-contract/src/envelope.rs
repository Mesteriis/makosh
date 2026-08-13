use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ObservationMetadataV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1, MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
    MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1, MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1,
    MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1,
    MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1, MAIL_OWNER_ID_V1, MAIL_PERSON_SOURCE_CAPABILITY_ID_V1,
    MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1, MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookContractV1,
    MailPersonSourceContractV1, validate_fetch_mail_person_source_page_v1,
    validate_mail_address_book_entry_observed_v1,
    validate_mail_address_book_entry_upsert_rejected_v1,
    validate_mail_address_book_entry_upserted_v1, validate_mail_address_book_page_completed_v1,
    validate_mail_address_book_page_rejected_v1, validate_mail_person_source_account_ready_v1,
    validate_mail_person_source_account_retired_v1, validate_mail_person_source_observed_v1,
    validate_mail_person_source_page_completed_v1, validate_mail_person_source_page_rejected_v1,
    validate_mail_person_source_removed_v1, validate_mail_person_source_updated_v1,
    wire::{
        FetchMailAddressBookPageCommandV1, MailAddressBookEntryObservedV1,
        MailAddressBookEntryUpsertRejectedV1, MailAddressBookEntryUpsertedV1,
        MailAddressBookPageCompletedV1, MailAddressBookPageRejectedV1,
        UpsertMailAddressBookEntryCommandV1,
    },
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourceAccountReadyV1,
        MailPersonSourceAccountRetiredV1, MailPersonSourceObservedV1,
        MailPersonSourcePageCompletedV1, MailPersonSourcePageRejectedV1, MailPersonSourceRemovedV1,
        MailPersonSourceUpdatedV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookResultEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub completed_at_unix_seconds: i64,
    pub completed_at_nanos: i32,
    pub execution_attempt: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_mail_person_source_account_ready_v1(
    causation_message_id: [u8; 16],
    payload: MailPersonSourceAccountReadyV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_account_ready_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_account_lifecycle_v1(
        causation_message_id,
        id16(&payload.account_event_id)?,
        id16(&payload.account_public_id)?,
        payload.mapping_revision,
        payload.observed_at.as_ref(),
        MailPersonSourceContractV1::AccountReady,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_person_source_account_retired_v1(
    causation_message_id: [u8; 16],
    payload: MailPersonSourceAccountRetiredV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_account_retired_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_account_lifecycle_v1(
        causation_message_id,
        id16(&payload.account_event_id)?,
        id16(&payload.account_public_id)?,
        payload.mapping_revision,
        payload.retired_at.as_ref(),
        MailPersonSourceContractV1::AccountRetired,
        payload.encode_to_vec(),
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_mail_person_source_account_lifecycle_v1(
    causation_message_id: [u8; 16],
    event_id: [u8; 16],
    account_public_id: [u8; 16],
    mapping_revision: u64,
    occurred_at: Option<&Timestamp>,
    contract: MailPersonSourceContractV1,
    payload: Vec<u8>,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let occurred_at = occurred_at
        .cloned()
        .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    if causation_message_id.iter().all(|byte| *byte == 0) || occurred_at != timestamp(context) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let mut cursor = Sha256::new();
    cursor.update(b"makosh.mail.person-source.account-lifecycle.v1");
    cursor.update(account_public_id);
    cursor.update(mapping_revision.to_be_bytes());
    cursor.update(contract.name().as_bytes());
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: event_id.to_vec(),
        contract: Some(person_source_contract_ref(contract)),
        source: Some(person_source_mail_source(context)),
        recorded_at: Some(occurred_at),
        partition_key: account_public_id.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: account_public_id.to_vec(),
        actor: Some(mail_actor()),
        trace: None,
        source_fence: Some(mail_fence(context)),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: event_id.to_vec(),
            observed_at: Some(occurred_at),
            occurred_at: Some(occurred_at),
            source_cursor_sha256: cursor.finalize().to_vec(),
            source_sequence: Some(mapping_revision),
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_fetch_mail_person_source_page_command_v1(
    payload: FetchMailPersonSourcePageCommandV1,
    deadline_unix_seconds: i64,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    validate_fetch_mail_person_source_page_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    let command_id = id16(&payload.command_id)?;
    let run_id = id16(&payload.run_id)?;
    if payload.page_size > MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let reference = MailPersonSourceContractV1::FetchPageCommand.reference();
    let payload_bytes = payload.encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: reference.owner,
            name: reference.name,
            major: reference.major,
            revision: reference.revision,
            schema_sha256: reference.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"mail-persons-sync-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-person-source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: MAIL_PERSON_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(&payload_bytes).to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload_bytes,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_mail_person_source_observed_v1(
    command_message_id: [u8; 16],
    payload: MailPersonSourceObservedV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_observed_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_observation_v1(
        command_message_id,
        id16(&payload.observation_id)?,
        id16(&payload.run_id)?,
        payload.page_sequence,
        payload
            .provenance
            .as_ref()
            .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?
            .source_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?,
        MailPersonSourceContractV1::SourceObserved,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_person_source_updated_v1(
    command_message_id: [u8; 16],
    payload: MailPersonSourceUpdatedV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_updated_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_observation_v1(
        command_message_id,
        id16(&payload.observation_id)?,
        id16(&payload.run_id)?,
        payload.page_sequence,
        payload
            .provenance
            .as_ref()
            .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?
            .source_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?,
        MailPersonSourceContractV1::SourceUpdated,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_person_source_removed_v1(
    command_message_id: [u8; 16],
    payload: MailPersonSourceRemovedV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_removed_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_observation_v1(
        command_message_id,
        id16(&payload.observation_id)?,
        id16(&payload.run_id)?,
        payload.page_sequence,
        payload
            .provenance
            .as_ref()
            .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?
            .source_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?,
        MailPersonSourceContractV1::SourceRemoved,
        payload.encode_to_vec(),
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_mail_person_source_observation_v1(
    command_message_id: [u8; 16],
    observation_id: [u8; 16],
    run_id: [u8; 16],
    page_sequence: u64,
    source_digest: [u8; 32],
    contract: MailPersonSourceContractV1,
    payload: Vec<u8>,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: observation_id.to_vec(),
        contract: Some(person_source_contract_ref(contract)),
        source: Some(person_source_mail_source(context)),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(mail_actor()),
        trace: None,
        source_fence: Some(mail_fence(context)),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: observation_id.to_vec(),
            observed_at: Some(timestamp(context)),
            occurred_at: Some(timestamp(context)),
            source_cursor_sha256: source_digest.to_vec(),
            source_sequence: Some(page_sequence),
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_mail_person_source_page_completed_v1(
    command_message_id: [u8; 16],
    payload: MailPersonSourcePageCompletedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_page_completed_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_result_v1(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailPersonSourceContractV1::PageCompleted,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_person_source_page_rejected_v1(
    command_message_id: [u8; 16],
    payload: MailPersonSourcePageRejectedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_person_source_page_rejected_v1(&payload)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    build_mail_person_source_result_v1(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailPersonSourceContractV1::PageRejected,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

fn build_mail_person_source_result_v1(
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    run_id: [u8; 16],
    contract: MailPersonSourceContractV1,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_result_context(context)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: digest16(
            b"mail-person-source-fetch-result-v1",
            &command_id,
            contract.name().as_bytes(),
        )
        .to_vec(),
        contract: Some(person_source_contract_ref(contract)),
        source: Some(SourceRefV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest16(
                b"mail-runtime-person-source-result-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-person-source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(mail_actor()),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(completed_at),
            execution_attempt: context.execution_attempt,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_fetch_mail_address_book_page_command_v1(
    payload: FetchMailAddressBookPageCommandV1,
    deadline_unix_seconds: i64,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let run_id = id16(&payload.run_id)?;
    if !valid_identity(&payload.logical_owner_id)
        || !valid_bounded(&payload.account_id, 256)
        || payload.page_sequence == 0
        || payload.page_size == 0
        || payload.page_size > MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1
        || payload.continuation_cursor.as_ref().is_some_and(|value| {
            value.is_empty() || value.len() > MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1
        })
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = MailAddressBookContractV1::FetchPageCommand;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: contract.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"mail-contacts-sync-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-address-book",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: digest16(
                b"mail-address-book-fetch-page-idempotency-v1",
                &run_id,
                &payload.page_size.to_be_bytes(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_mail_address_book_entry_observed_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookEntryObservedV1,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    validate_mail_address_book_entry_observed_v1(&payload)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let observation_id = id16(&payload.observation_id)?;
    let run_id = id16(&payload.run_id)?;
    let observed_at = payload
        .observed_at
        .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    let source_cursor_sha256: [u8; 32] = payload
        .entry_digest
        .as_slice()
        .try_into()
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: observation_id.to_vec(),
        contract: Some(contract_ref(MailAddressBookContractV1::EntryObserved)),
        source: Some(mail_source(context)),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(mail_actor()),
        trace: None,
        source_fence: Some(mail_fence(context)),
        semantics: Some(Semantics::Observation(ObservationMetadataV1 {
            observation_id: observation_id.to_vec(),
            observed_at: Some(timestamp(context)),
            occurred_at: Some(observed_at),
            source_cursor_sha256: source_cursor_sha256.to_vec(),
            source_sequence: Some(payload.page_sequence),
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_mail_address_book_page_completed_result_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookPageCompletedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_address_book_page_completed_v1(&payload)?;
    build_fetch_result(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailAddressBookContractV1::PageCompleted,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_address_book_page_rejected_result_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookPageRejectedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_address_book_page_rejected_v1(&payload)?;
    build_fetch_result(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailAddressBookContractV1::PageRejected,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

fn build_fetch_result(
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    run_id: [u8; 16],
    contract: MailAddressBookContractV1,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_result_context(context)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: digest16(
            b"mail-address-book-fetch-result-v1",
            &command_id,
            contract.name().as_bytes(),
        )
        .to_vec(),
        contract: Some(contract_ref(contract)),
        source: Some(SourceRefV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest16(
                b"mail-runtime-address-book-fetch-source-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-address-book",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(Timestamp {
                seconds: context.completed_at_unix_seconds,
                nanos: context.completed_at_nanos,
            }),
            execution_attempt: context.execution_attempt,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn contract_ref(contract: MailAddressBookContractV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: MAIL_OWNER_ID_V1.to_owned(),
        name: contract.name().to_owned(),
        major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
        revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
        schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn person_source_contract_ref(contract: MailPersonSourceContractV1) -> ContractRefV1 {
    let reference = contract.reference();
    ContractRefV1 {
        owner: reference.owner,
        name: reference.name,
        major: reference.major,
        revision: reference.revision,
        schema_sha256: reference.schema_sha256,
    }
}

fn person_source_mail_source(context: &MailAddressBookEnvelopeContextV1) -> SourceRefV1 {
    SourceRefV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: digest16(
            b"mail-runtime-person-source-observation-v1",
            context.runtime_instance_id.as_bytes(),
            b"mail-person-source",
        )
        .to_vec(),
        runtime_generation: context.runtime_generation,
    }
}

fn mail_source(context: &MailAddressBookEnvelopeContextV1) -> SourceRefV1 {
    SourceRefV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: digest16(
            b"mail-runtime-address-book-observation-source-v1",
            context.runtime_instance_id.as_bytes(),
            b"mail-address-book",
        )
        .to_vec(),
        runtime_generation: context.runtime_generation,
    }
}

fn mail_actor() -> ActorRefV1 {
    ActorRefV1 {
        kind: ActorKindV1::Module as i32,
        actor_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
    }
}

fn mail_fence(context: &MailAddressBookEnvelopeContextV1) -> SourceFenceV1 {
    SourceFenceV1 {
        kind: FenceKindV1::RuntimeLease as i32,
        scope_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
        epoch: context.runtime_generation,
    }
}

pub fn build_upsert_mail_address_book_entry_command_v1(
    payload: UpsertMailAddressBookEntryCommandV1,
    deadline_unix_seconds: i64,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let run_id = id16(&payload.run_id)?;
    id16(&payload.contact_snapshot_reference_id)?;
    if !valid_identity(&payload.logical_owner_id)
        || !valid_bounded(&payload.account_id, 256)
        || payload.contact_snapshot_sha256.len() != 32
        || payload
            .contact_snapshot_sha256
            .iter()
            .all(|byte| *byte == 0)
        || payload.expected_contact_revision == 0
        || payload.contact_snapshot_declared_bytes == 0
        || payload.contact_snapshot_declared_bytes > 32 * 1024
        || payload.contact_snapshot_custody_source_proof.is_empty()
        || payload.contact_snapshot_custody_source_proof.len()
            > MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = MailAddressBookContractV1::UpsertEntryCommand;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: contract.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"mail-contacts-sync-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-address-book-write",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: run_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: digest16(
                b"mail-address-book-upsert-entry-idempotency-v1",
                &run_id,
                &payload.expected_contact_revision.to_be_bytes(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

pub fn build_mail_address_book_entry_upserted_result_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookEntryUpsertedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_address_book_entry_upserted_v1(&payload)?;
    build_upsert_result(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailAddressBookContractV1::EntryUpserted,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_mail_address_book_entry_upsert_rejected_result_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookEntryUpsertRejectedV1,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_mail_address_book_entry_upsert_rejected_v1(&payload)?;
    let outcome = if payload.outcome_unknown {
        ResultOutcomeV1::Failed
    } else {
        ResultOutcomeV1::Rejected
    };
    build_upsert_result(
        command_message_id,
        id16(&payload.command_id)?,
        id16(&payload.run_id)?,
        MailAddressBookContractV1::EntryUpsertRejected,
        outcome,
        payload.encode_to_vec(),
        context,
    )
}

fn build_upsert_result(
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    run_id: [u8; 16],
    contract: MailAddressBookContractV1,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailAddressBookEnvelopeBuildErrorV1> {
    validate_result_context(context)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: digest16(
            b"mail-address-book-upsert-result-v1",
            &command_id,
            contract.name().as_bytes(),
        )
        .to_vec(),
        contract: Some(ContractRefV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: contract.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest16(
                b"mail-runtime-address-book-result-source-v1",
                context.runtime_instance_id.as_bytes(),
                b"mail-address-book",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_RUNTIME_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(Timestamp {
                seconds: context.completed_at_unix_seconds,
                nanos: context.completed_at_nanos,
            }),
            execution_attempt: context.execution_attempt,
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_result_context(
    context: &MailAddressBookResultEnvelopeContextV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    if !valid_bounded(&context.runtime_instance_id, 128)
        || context.runtime_generation == 0
        || context.completed_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.completed_at_nanos)
        || context.execution_attempt == 0
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_context(
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    if !valid_bounded(&context.module_id, 128)
        || !valid_bounded(&context.runtime_instance_id, 128)
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn timestamp(context: &MailAddressBookEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailAddressBookEnvelopeBuildErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    if value.iter().all(|byte| *byte == 0) {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn valid_identity(value: &str) -> bool {
    valid_bounded(value, 128)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_bounded(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.is_ascii() && value.trim() == value
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update([0]);
    digest.update(left);
    digest.update([0]);
    digest.update(right);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

const fn outbox_error(_: OutboxRecordError) -> MailAddressBookEnvelopeBuildErrorV1 {
    MailAddressBookEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::validation::envelope::decode_envelope_v1;

    use super::*;
    use crate::MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1;

    #[test]
    fn account_ready_observation_is_exactly_public_and_partitioned() {
        let payload = MailPersonSourceAccountReadyV1 {
            account_event_id: vec![0x11; 16],
            logical_owner_id: "owner-1".to_owned(),
            integration_public_id: vec![0x12; 16],
            account_public_id: vec![0x13; 16],
            mapping_revision: 1,
            observed_at: Some(Timestamp {
                seconds: 1_700_000_000,
                nanos: 7,
            }),
        };
        let record = build_mail_person_source_account_ready_v1(
            [0x21; 16],
            payload,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 3,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 7,
            },
        )
        .expect("ready observation");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.partition_key, vec![0x13; 16]);
        assert_eq!(envelope.correlation_id, vec![0x13; 16]);
        assert_eq!(envelope.causation_message_id, vec![0x21; 16]);
        assert_eq!(
            envelope.contract.expect("contract").name,
            MailPersonSourceContractV1::AccountReady.name()
        );
        let Semantics::Observation(metadata) = envelope.semantics.expect("semantics") else {
            panic!("observation semantics");
        };
        assert_eq!(metadata.source_sequence, Some(1));
    }

    #[test]
    fn fetch_command_is_mail_targeted_and_provider_neutral() {
        let record = build_fetch_mail_address_book_page_command_v1(
            FetchMailAddressBookPageCommandV1 {
                command_id: vec![1; 16],
                run_id: vec![2; 16],
                logical_owner_id: "owner-1".to_owned(),
                account_id: "mail-account-1".to_owned(),
                page_sequence: 1,
                continuation_cursor: None,
                page_size: 100,
            },
            1_800_000_030,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("command");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.contract.expect("contract").owner, MAIL_OWNER_ID_V1);
        assert_eq!(envelope.partition_key, vec![2; 16]);
    }

    #[test]
    fn upsert_terminal_result_is_correlated_to_exact_command() {
        let record = build_mail_address_book_entry_upserted_result_v1(
            [3; 16],
            MailAddressBookEntryUpsertedV1 {
                command_id: vec![1; 16],
                run_id: vec![2; 16],
                provider_entry_id: "people/abc".to_owned(),
                provider_etag: "etag-1".to_owned(),
                applied_contact_revision: 7,
                provider_kind: crate::wire::MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople as i32,
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 4,
                completed_at_unix_seconds: 1_700_000_100,
                completed_at_nanos: 5,
                execution_attempt: 1,
            },
        )
        .expect("result");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.partition_key, vec![2; 16]);
        assert_eq!(envelope.correlation_id, vec![2; 16]);
        assert_eq!(envelope.causation_message_id, vec![3; 16]);
        assert_eq!(
            envelope.contract.expect("contract").name,
            MailAddressBookContractV1::EntryUpserted.name(),
        );
        let Semantics::Result(metadata) = envelope.semantics.expect("semantics") else {
            panic!("result semantics");
        };
        assert_eq!(metadata.command_id, vec![1; 16]);
        assert_eq!(metadata.command_message_id, vec![3; 16]);
        assert_eq!(metadata.outcome, ResultOutcomeV1::Succeeded as i32);
    }

    #[test]
    fn outcome_unknown_rejection_requires_exact_code_pairing() {
        let invalid = MailAddressBookEntryUpsertRejectedV1 {
            command_id: vec![1; 16],
            run_id: vec![2; 16],
            code: crate::wire::MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
                as i32,
            outcome_unknown: true,
        };
        assert_eq!(
            validate_mail_address_book_entry_upsert_rejected_v1(&invalid),
            Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload),
        );
    }
}
