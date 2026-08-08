use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_CONTRACT_NAME_V1,
    BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_REJECTED_CONTRACT_NAME_V1,
    CONTACT_UPSERT_FROM_MAIL_ENTRY_REJECTED_CONTRACT_NAME_V1,
    CONTACT_UPSERTED_FROM_MAIL_ENTRY_CONTRACT_NAME_V1, CONTACTS_COMMAND_CONTRACT_MAJOR_V1,
    CONTACTS_COMMAND_CONTRACT_REVISION_V1, CONTACTS_COMMAND_SCHEMA_SHA256_V1,
    CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1,
    CONTACTS_MAIL_PROVIDER_LINK_COMMAND_CAPABILITY_ID_V1, CONTACTS_OWNER_ID_V1,
    MAIL_ADDRESS_BOOK_PROVIDER_LINK_BOUND_CONTRACT_NAME_V1,
    UPSERT_CONTACT_FROM_MAIL_ENTRY_CONTRACT_NAME_V1,
    wire::{
        BindMailAddressBookProviderLinkCommandV1, BindMailAddressBookProviderLinkRejectedV1,
        ContactUpsertFromMailAddressBookEntryRejectedV1, ContactUpsertedFromMailAddressBookEntryV1,
        MailAddressBookProviderLinkBoundV1, UpsertContactFromMailAddressBookEntryCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactsCommandEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsCommandEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

struct EnvelopeBuildV1<'a> {
    message_id: [u8; 16],
    partition_key: [u8; 16],
    causation_message_id: Vec<u8>,
    contract_name: &'a str,
    semantics: Semantics,
    payload: Vec<u8>,
}

struct ResultBuildV1<'a> {
    label: &'a [u8],
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    partition_key: [u8; 16],
    contract_name: &'a str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
}

pub fn build_upsert_contact_command_outbox_record_v1(
    payload: UpsertContactFromMailAddressBookEntryCommandV1,
    deadline_unix_seconds: i64,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = validate_command(&payload)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    let partition_key = digest16(
        b"contacts-mail-entry-partition-v1",
        payload.logical_owner_id.as_bytes(),
        payload.source_account_id.as_bytes(),
    );
    build(
        EnvelopeBuildV1 {
            message_id: command_id,
            partition_key,
            causation_message_id: Vec::new(),
            contract_name: UPSERT_CONTACT_FROM_MAIL_ENTRY_CONTRACT_NAME_V1,
            semantics: Semantics::Command(CommandMetadataV1 {
                command_id: command_id.to_vec(),
                target_capability: CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1.to_owned(),
                idempotency_key: digest16(
                    b"contacts-mail-entry-idempotency-v1",
                    payload.logical_owner_id.as_bytes(),
                    &payload.entry_digest,
                )
                .to_vec(),
                deadline: Some(Timestamp {
                    seconds: deadline_unix_seconds,
                    nanos: 0,
                }),
                logical_attempt: 1,
            }),
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_contact_upserted_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: ContactUpsertedFromMailAddressBookEntryV1,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let contact_id = id16(&payload.contact_id)?;
    if !nonzero(&command_message_id)
        || payload.contact_revision == 0
        || payload.outcome == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        ResultBuildV1 {
            label: b"contacts-mail-entry-upserted-v1",
            command_message_id,
            command_id,
            partition_key: contact_id,
            contract_name: CONTACT_UPSERTED_FROM_MAIL_ENTRY_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Succeeded,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_contact_upsert_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: ContactUpsertFromMailAddressBookEntryRejectedV1,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    if !nonzero(&command_message_id) || payload.code == 0 || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        ResultBuildV1 {
            label: b"contacts-mail-entry-rejected-v1",
            command_message_id,
            command_id,
            partition_key: command_id,
            contract_name: CONTACT_UPSERT_FROM_MAIL_ENTRY_REJECTED_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Rejected,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_bind_mail_address_book_provider_link_command_outbox_record_v1(
    causation_message_id: [u8; 16],
    payload: BindMailAddressBookProviderLinkCommandV1,
    deadline_unix_seconds: i64,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = validate_bind_provider_link_command(&payload)?;
    id16(&causation_message_id)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contact_id = id16(&payload.contact_id)?;
    build(
        EnvelopeBuildV1 {
            message_id: command_id,
            partition_key: contact_id,
            causation_message_id: causation_message_id.to_vec(),
            contract_name: BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_CONTRACT_NAME_V1,
            semantics: Semantics::Command(CommandMetadataV1 {
                command_id: command_id.to_vec(),
                target_capability: CONTACTS_MAIL_PROVIDER_LINK_COMMAND_CAPABILITY_ID_V1.to_owned(),
                idempotency_key: digest16(
                    b"contacts-mail-provider-link-idempotency-v1",
                    &contact_id,
                    payload.source_account_id.as_bytes(),
                )
                .to_vec(),
                deadline: Some(Timestamp {
                    seconds: deadline_unix_seconds,
                    nanos: 0,
                }),
                logical_attempt: 1,
            }),
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_mail_address_book_provider_link_bound_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: MailAddressBookProviderLinkBoundV1,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let contact_id = id16(&payload.contact_id)?;
    if !nonzero(&command_message_id)
        || payload.contact_revision == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        ResultBuildV1 {
            label: b"contacts-mail-provider-link-bound-v1",
            command_message_id,
            command_id,
            partition_key: contact_id,
            contract_name: MAIL_ADDRESS_BOOK_PROVIDER_LINK_BOUND_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Succeeded,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

pub fn build_bind_mail_address_book_provider_link_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: BindMailAddressBookProviderLinkRejectedV1,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    if !nonzero(&command_message_id) || payload.code == 0 || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        ResultBuildV1 {
            label: b"contacts-mail-provider-link-rejected-v1",
            command_message_id,
            command_id,
            partition_key: command_id,
            contract_name: BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_REJECTED_CONTRACT_NAME_V1,
            outcome: ResultOutcomeV1::Rejected,
            payload: payload.encode_to_vec(),
        },
        context,
    )
}

fn result(
    input: ResultBuildV1<'_>,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    build(
        EnvelopeBuildV1 {
            message_id: digest16(input.label, &input.command_id, &input.partition_key),
            partition_key: input.partition_key,
            causation_message_id: input.command_message_id.to_vec(),
            contract_name: input.contract_name,
            semantics: Semantics::Result(ResultMetadataV1 {
                command_id: input.command_id.to_vec(),
                command_message_id: input.command_message_id.to_vec(),
                outcome: input.outcome as i32,
                completed_at: Some(timestamp(context)),
                execution_attempt: 1,
            }),
            payload: input.payload,
        },
        context,
    )
}

fn build(
    input: EnvelopeBuildV1<'_>,
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsCommandEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: input.message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: CONTACTS_OWNER_ID_V1.to_owned(),
            name: input.contract_name.to_owned(),
            major: CONTACTS_COMMAND_CONTRACT_MAJOR_V1,
            revision: CONTACTS_COMMAND_CONTRACT_REVISION_V1,
            schema_sha256: CONTACTS_COMMAND_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"contacts-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: input.partition_key.to_vec(),
        causation_message_id: input.causation_message_id,
        correlation_id: input.partition_key.to_vec(),
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
        semantics: Some(input.semantics),
        payload: input.payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ContactsCommandEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_command(
    payload: &UpsertContactFromMailAddressBookEntryCommandV1,
) -> Result<[u8; 16], ContactsCommandEnvelopeBuildErrorV1> {
    let command_id = id16(&payload.command_id)?;
    id32(&payload.entry_digest)?;
    let valid_provider = matches!(payload.provider_kind, 1 | 2);
    let valid_timestamp = payload
        .observed_at
        .as_ref()
        .is_some_and(|value| value.seconds > 0 && (0..1_000_000_000).contains(&value.nanos));
    if !valid_owner(&payload.logical_owner_id)
        || !valid_bounded(&payload.source_account_id, 256, false)
        || !valid_provider
        || !valid_bounded(&payload.provider_entry_id, 512, false)
        || payload
            .provider_etag
            .as_deref()
            .is_some_and(|value| !valid_bounded(value, 512, false))
        || !valid_bounded(&payload.display_name, 240, true)
        || payload.email_addresses.len() > 32
        || payload.phone_numbers.len() > 32
        || payload
            .email_addresses
            .iter()
            .any(|value| !valid_bounded(value, 320, false))
        || payload
            .phone_numbers
            .iter()
            .any(|value| !valid_bounded(value, 32, false))
        || (payload.email_addresses.is_empty()
            && payload.phone_numbers.is_empty()
            && payload.display_name.trim().is_empty())
        || !valid_timestamp
        || payload.source_revision == 0
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(command_id)
}

fn validate_bind_provider_link_command(
    payload: &BindMailAddressBookProviderLinkCommandV1,
) -> Result<[u8; 16], ContactsCommandEnvelopeBuildErrorV1> {
    let command_id = id16(&payload.command_id)?;
    id16(&payload.contact_id)?;
    if !valid_owner(&payload.logical_owner_id)
        || payload.expected_contact_revision == 0
        || !valid_bounded(&payload.source_account_id, 256, false)
        || !matches!(payload.provider_kind, 1 | 2)
        || !valid_bounded(&payload.provider_entry_id, 512, false)
        || payload
            .provider_etag
            .as_deref()
            .is_some_and(|value| !valid_bounded(value, 512, false))
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(command_id)
}

fn validate_context(
    context: &ContactsCommandEnvelopeContextV1,
) -> Result<(), ContactsCommandEnvelopeBuildErrorV1> {
    if !valid_bounded(&context.module_id, 128, false)
        || !valid_bounded(&context.runtime_instance_id, 128, false)
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], ContactsCommandEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| ContactsCommandEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&value) {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn id32(bytes: &[u8]) -> Result<[u8; 32], ContactsCommandEnvelopeBuildErrorV1> {
    let value: [u8; 32] = bytes
        .try_into()
        .map_err(|_| ContactsCommandEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&value) {
        return Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_bounded(value: &str, max_bytes: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.len() <= max_bytes
        && !value.chars().any(char::is_control)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn timestamp(context: &ContactsCommandEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update((first.len() as u64).to_be_bytes());
    hasher.update(first);
    hasher.update((second.len() as u64).to_be_bytes());
    hasher.update(second);
    hasher.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox_error(_: OutboxRecordError) -> ContactsCommandEnvelopeBuildErrorV1 {
    ContactsCommandEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        BindMailAddressBookProviderLinkCommandV1, MailAddressBookProviderKindV1,
        UpsertContactFromMailAddressBookEntryCommandV1,
    };

    fn context() -> ContactsCommandEnvelopeContextV1 {
        ContactsCommandEnvelopeContextV1 {
            module_id: "makosh-mail-contacts-sync-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 3,
        }
    }

    fn command() -> UpsertContactFromMailAddressBookEntryCommandV1 {
        UpsertContactFromMailAddressBookEntryCommandV1 {
            command_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source_account_id: "mail-account-1".to_owned(),
            provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail as i32,
            provider_entry_id: "people/c123".to_owned(),
            provider_etag: Some("etag-1".to_owned()),
            display_name: "Ada Lovelace".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: vec!["+34910000000".to_owned()],
            observed_at: Some(Timestamp {
                seconds: 1_800_000_000,
                nanos: 2,
            }),
            source_revision: 4,
            entry_digest: vec![2; 32],
        }
    }

    #[test]
    fn command_builds_exact_contacts_owned_envelope() {
        let record =
            build_upsert_contact_command_outbox_record_v1(command(), 1_800_000_030, &context())
                .expect("outbox");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode");
        assert_eq!(
            envelope.contract.expect("contract").owner,
            CONTACTS_OWNER_ID_V1
        );
    }

    #[test]
    fn empty_identity_and_name_is_rejected() {
        let mut invalid = command();
        invalid.display_name.clear();
        invalid.email_addresses.clear();
        invalid.phone_numbers.clear();
        assert_eq!(
            build_upsert_contact_command_outbox_record_v1(invalid, 1_800_000_030, &context()),
            Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn provider_link_command_is_contacts_owned_and_contact_partitioned() {
        let payload = BindMailAddressBookProviderLinkCommandV1 {
            command_id: vec![3; 16],
            logical_owner_id: "owner-1".to_owned(),
            contact_id: vec![4; 16],
            expected_contact_revision: 8,
            source_account_id: "mail-account-1".to_owned(),
            provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail as i32,
            provider_entry_id: "people/created-contact-1".to_owned(),
            provider_etag: Some("created-etag-1".to_owned()),
        };
        let record = build_bind_mail_address_book_provider_link_command_outbox_record_v1(
            [2; 16],
            payload,
            1_800_000_030,
            &context(),
        )
        .expect("provider link command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode");
        let contract = envelope.contract.expect("contract");
        assert_eq!(contract.owner, CONTACTS_OWNER_ID_V1);
        assert_eq!(
            contract.name,
            BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_CONTRACT_NAME_V1
        );
        assert_eq!(envelope.partition_key, vec![4; 16]);
    }

    #[test]
    fn provider_link_command_rejects_unspecified_provider() {
        let payload = BindMailAddressBookProviderLinkCommandV1 {
            command_id: vec![3; 16],
            logical_owner_id: "owner-1".to_owned(),
            contact_id: vec![4; 16],
            expected_contact_revision: 8,
            source_account_id: "mail-account-1".to_owned(),
            provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindUnspecified
                as i32,
            provider_entry_id: "people/created-contact-1".to_owned(),
            provider_etag: None,
        };
        assert_eq!(
            build_bind_mail_address_book_provider_link_command_outbox_record_v1(
                [2; 16],
                payload,
                1_800_000_030,
                &context(),
            ),
            Err(ContactsCommandEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
