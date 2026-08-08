//! Durable Mail address-book command intake.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1, MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1,
    MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1, MailAddressBookContractV1,
    wire::{FetchMailAddressBookPageCommandV1, UpsertMailAddressBookEntryCommandV1},
};
use makosh_mail_address_book_persistence::{
    MailAddressBookCommandInboxOutcomeV1, MailAddressBookFetchAdmissionV1,
    MailAddressBookFetchInboxOutcomeV1, MailAddressBookPersistenceV1,
    MailAddressBookUpsertAdmissionV1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookConsumeErrorV1 {
    Unavailable,
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongAudience,
    OwnerMismatch,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_mail_address_book_fetch_v1(
    persistence: &MailAddressBookPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    accepted_at_unix_seconds: i64,
) -> Result<MailAddressBookFetchInboxOutcomeV1, MailAddressBookConsumeErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Unavailable)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidEnvelope)?;
    let admission = decode_fetch(&record, expected_logical_owner_id)?;
    let outcome = persistence
        .accept_fetch_command(&admission, accepted_at_unix_seconds)
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Unavailable)?;
    Ok(outcome)
}

pub fn decode_fetch(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<MailAddressBookFetchAdmissionV1, MailAddressBookConsumeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidEnvelope)?;
    validate_command_envelope(
        &envelope,
        MailAddressBookContractV1::FetchPageCommand,
        expected_logical_owner_id,
    )?;
    let command = FetchMailAddressBookPageCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidPayload)?;
    let command_id = id16(&command.command_id)?;
    let run_id = id16(&command.run_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookConsumeErrorV1::WrongContract);
    };
    if command.logical_owner_id != expected_logical_owner_id
        || command_message_id != command_id
        || metadata.command_id.as_slice() != command_id
        || envelope.partition_key.as_slice() != run_id
        || envelope.correlation_id.as_slice() != run_id
        || command.account_id.trim().is_empty()
        || command.account_id.len() > 256
        || command.page_sequence == 0
        || !(1..=500).contains(&command.page_size)
        || command
            .continuation_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.is_empty() || cursor.len() > 4096)
    {
        return Err(MailAddressBookConsumeErrorV1::InvalidPayload);
    }
    Ok(MailAddressBookFetchAdmissionV1 {
        command_message_id,
        command_envelope_sha256: *record.envelope_sha256(),
        command_id,
        run_id,
        logical_owner_id: command.logical_owner_id,
        account_id: command.account_id,
        page_sequence: command.page_sequence,
        continuation_cursor: command.continuation_cursor,
        page_size: command.page_size,
    })
}

pub async fn consume_next_mail_address_book_upsert_v1(
    persistence: &MailAddressBookPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    accepted_at_unix_seconds: i64,
) -> Result<MailAddressBookCommandInboxOutcomeV1, MailAddressBookConsumeErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Unavailable)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidEnvelope)?;
    let admission = decode_upsert(&record, expected_logical_owner_id)?;
    let outcome = persistence
        .accept_upsert_command(&admission, accepted_at_unix_seconds)
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailAddressBookConsumeErrorV1::Unavailable)?;
    Ok(outcome)
}

pub fn decode_upsert(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<MailAddressBookUpsertAdmissionV1, MailAddressBookConsumeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidEnvelope)?;
    validate_command_envelope(
        &envelope,
        MailAddressBookContractV1::UpsertEntryCommand,
        expected_logical_owner_id,
    )?;
    let command = UpsertMailAddressBookEntryCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidPayload)?;
    if command.logical_owner_id != expected_logical_owner_id {
        return Err(MailAddressBookConsumeErrorV1::OwnerMismatch);
    }
    let command_id = id16(&command.command_id)?;
    let run_id = id16(&command.run_id)?;
    let command_message_id = id16(&envelope.message_id)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookConsumeErrorV1::WrongContract);
    };
    if command_message_id != command_id
        || metadata.command_id.as_slice() != command_id
        || envelope.partition_key.as_slice() != run_id
        || envelope.correlation_id.as_slice() != run_id
        || command.logical_owner_id.trim().is_empty()
        || command.logical_owner_id.len() > 256
        || command.account_id.trim().is_empty()
        || command.account_id.len() > 256
        || command.expected_contact_revision == 0
        || !(1..=32 * 1024).contains(&command.contact_snapshot_declared_bytes)
        || command.contact_snapshot_custody_source_proof.is_empty()
        || command.contact_snapshot_custody_source_proof.len()
            > MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1
    {
        return Err(MailAddressBookConsumeErrorV1::InvalidPayload);
    }
    Ok(MailAddressBookUpsertAdmissionV1 {
        command_message_id,
        command_envelope_sha256: *record.envelope_sha256(),
        command_id,
        run_id,
        logical_owner_id: command.logical_owner_id,
        account_id: command.account_id,
        contact_snapshot_reference_id: id16(&command.contact_snapshot_reference_id)?,
        contact_snapshot_sha256: id32(&command.contact_snapshot_sha256)?,
        expected_contact_revision: command.expected_contact_revision,
        contact_snapshot_declared_bytes: command.contact_snapshot_declared_bytes,
        contact_snapshot_custody_source_proof: command.contact_snapshot_custody_source_proof,
    })
}

fn validate_command_envelope(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    expected_contract: MailAddressBookContractV1,
    expected_logical_owner_id: &str,
) -> Result<(), MailAddressBookConsumeErrorV1> {
    if expected_logical_owner_id.trim().is_empty() || expected_logical_owner_id.len() > 128 {
        return Err(MailAddressBookConsumeErrorV1::OwnerMismatch);
    }
    let expected = expected_contract.reference();
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(MailAddressBookConsumeErrorV1::WrongContract)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
    {
        return Err(MailAddressBookConsumeErrorV1::WrongContract);
    }
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookConsumeErrorV1::WrongContract);
    };
    if metadata.target_capability != MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1 {
        return Err(MailAddressBookConsumeErrorV1::WrongAudience);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailAddressBookConsumeErrorV1::WrongSource)?;
    if source.module_id != MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1
        || source.runtime_generation == 0
        || envelope.source_fence.as_ref().is_none_or(|fence| {
            fence.scope_id != MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1.as_bytes()
                || fence.epoch != source.runtime_generation
        })
    {
        return Err(MailAddressBookConsumeErrorV1::WrongSource);
    }
    let _ = metadata;
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailAddressBookConsumeErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(MailAddressBookConsumeErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], MailAddressBookConsumeErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| MailAddressBookConsumeErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(MailAddressBookConsumeErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{
        delivery::OutboxRecordV1, validation::envelope::decode_envelope_v1,
    };
    use makosh_mail_address_book_contract::{
        MailAddressBookEnvelopeContextV1, build_fetch_mail_address_book_page_command_v1,
        build_upsert_mail_address_book_entry_command_v1,
    };

    use super::*;

    fn command_record() -> OutboxRecordV1 {
        build_upsert_mail_address_book_entry_command_v1(
            UpsertMailAddressBookEntryCommandV1 {
                command_id: vec![1; 16],
                run_id: vec![2; 16],
                logical_owner_id: "owner-1".to_owned(),
                account_id: "mail-1".to_owned(),
                contact_snapshot_reference_id: vec![3; 16],
                contact_snapshot_sha256: vec![4; 32],
                expected_contact_revision: 7,
                contact_snapshot_declared_bytes: 128,
                contact_snapshot_custody_source_proof: vec![5; 32],
            },
            200,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "workflow-runtime-1".to_owned(),
                runtime_generation: 3,
                recorded_at_unix_seconds: 100,
                recorded_at_nanos: 0,
            },
        )
        .expect("command")
    }

    #[test]
    fn decoder_accepts_only_exact_workflow_source_audience_and_owner() {
        let record = command_record();
        let decoded = decode_upsert(&record, "owner-1").expect("decoded");
        assert_eq!(decoded.command_id, [1; 16]);
        assert_eq!(decoded.run_id, [2; 16]);
        assert_eq!(decoded.account_id, "mail-1");
        assert_eq!(decoded.expected_contact_revision, 7);
        assert_eq!(
            decode_upsert(&record, "owner-2"),
            Err(MailAddressBookConsumeErrorV1::OwnerMismatch)
        );

        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(metadata)) = envelope.semantics.as_mut() else {
            panic!("command semantics")
        };
        metadata.target_capability = "mail.wrong.v1".to_owned();
        let wrong = OutboxRecordV1::accept(envelope.encode_to_vec()).expect("valid envelope");
        assert_eq!(
            decode_upsert(&wrong, "owner-1"),
            Err(MailAddressBookConsumeErrorV1::WrongAudience)
        );
    }

    #[test]
    fn fetch_decoder_preserves_opaque_page_state_and_exact_sequence() {
        let record = build_fetch_mail_address_book_page_command_v1(
            FetchMailAddressBookPageCommandV1 {
                command_id: vec![7; 16],
                run_id: vec![8; 16],
                logical_owner_id: "owner-1".to_owned(),
                account_id: "mail-1".to_owned(),
                page_sequence: 3,
                continuation_cursor: Some(b"provider-owned-cursor".to_vec()),
                page_size: 50,
            },
            200,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "workflow-runtime-1".to_owned(),
                runtime_generation: 3,
                recorded_at_unix_seconds: 100,
                recorded_at_nanos: 0,
            },
        )
        .expect("fetch command");
        let decoded = decode_fetch(&record, "owner-1").expect("decoded fetch");
        assert_eq!(decoded.command_id, [7; 16]);
        assert_eq!(decoded.run_id, [8; 16]);
        assert_eq!(decoded.page_sequence, 3);
        assert_eq!(
            decoded.continuation_cursor.as_deref(),
            Some(b"provider-owned-cursor".as_slice())
        );
    }
}
