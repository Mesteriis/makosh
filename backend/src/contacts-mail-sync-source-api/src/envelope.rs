use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1,
        EventMetadataV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    CONTACT_CHANGED_FOR_MAIL_SYNC_CONTRACT_NAME_V1, CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1,
    CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1, CONTACT_MAIL_SYNC_SOURCE_PREPARE_CONTRACT_NAME_V1,
    CONTACT_MAIL_SYNC_SOURCE_PREPARED_CONTRACT_NAME_V1,
    CONTACT_MAIL_SYNC_SOURCE_REJECTED_CONTRACT_NAME_V1, CONTACTS_MAIL_SYNC_SOURCE_CAPABILITY_ID_V1,
    CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_MAJOR_V1, CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_REVISION_V1,
    CONTACTS_MAIL_SYNC_SOURCE_OWNER_V1, CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_SHA256_V1,
    wire::{
        ContactChangedForMailSyncV1, ContactMailSyncSourcePreparedV1,
        ContactMailSyncSourceRejectedV1, PrepareContactMailSyncSourceCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactsMailSyncSourceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsMailSyncSourceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_contact_changed_for_mail_sync_outbox_record_v1(
    payload: ContactChangedForMailSyncV1,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    build_contact_changed(payload, None, context)
}

pub fn build_contact_changed_for_mail_sync_outbox_record_caused_by_v1(
    command_message_id: [u8; 16],
    payload: ContactChangedForMailSyncV1,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    id16(&command_message_id)?;
    build_contact_changed(payload, Some(command_message_id), context)
}

fn build_contact_changed(
    payload: ContactChangedForMailSyncV1,
    causation_message_id: Option<[u8; 16]>,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let contact_id = id16(&payload.contact_id)?;
    if payload.contact_revision == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let message_id = digest16(
        b"contacts-mail-sync-changed-v1",
        &contact_id,
        &payload.contact_revision.to_be_bytes(),
    );
    build(
        message_id,
        contact_id,
        causation_message_id.map_or_else(Vec::new, |value| value.to_vec()),
        CONTACT_CHANGED_FOR_MAIL_SYNC_CONTRACT_NAME_V1,
        Semantics::Event(EventMetadataV1 {
            occurred_at: Some(timestamp(context)),
        }),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_contact_mail_sync_source_prepare_outbox_record_v1(
    payload: PrepareContactMailSyncSourceCommandV1,
    deadline_unix_seconds: i64,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let operation_id = id16(&payload.operation_id)?;
    let contact_id = id16(&payload.contact_id)?;
    if payload.expected_contact_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || !valid_bounded(&payload.target_mail_account_id, 256)
        || deadline_unix_seconds <= context.recorded_at_unix_seconds
    {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let idempotency_key = digest32(
        b"contacts-mail-sync-source-prepare-v1",
        &operation_id,
        &[
            contact_id.as_slice(),
            &payload.expected_contact_revision.to_be_bytes(),
            payload.target_mail_account_id.as_bytes(),
        ]
        .concat(),
    );
    build(
        operation_id,
        operation_id,
        Vec::new(),
        CONTACT_MAIL_SYNC_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: CONTACTS_MAIL_SYNC_SOURCE_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: idempotency_key.to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_contact_mail_sync_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: ContactMailSyncSourcePreparedV1,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let operation_id = validate_prepared(&payload)?;
    id16(&command_message_id)?;
    build_result(
        b"contacts-mail-sync-source-prepared-v1",
        command_message_id,
        operation_id,
        CONTACT_MAIL_SYNC_SOURCE_PREPARED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_contact_mail_sync_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: ContactMailSyncSourceRejectedV1,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let operation_id = id16(&payload.operation_id)?;
    id16(&command_message_id)?;
    if payload.code == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_result(
        b"contacts-mail-sync-source-rejected-v1",
        command_message_id,
        operation_id,
        CONTACT_MAIL_SYNC_SOURCE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

fn validate_prepared(
    payload: &ContactMailSyncSourcePreparedV1,
) -> Result<[u8; 16], ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    let operation_id = id16(&payload.operation_id)?;
    id16(&payload.contact_id)?;
    if payload.contact_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || payload.source_content.as_ref().is_none_or(|receipt| {
            id16(&receipt.reference_id).is_err()
                || receipt.declared_bytes == 0
                || receipt.declared_bytes > CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1
                || receipt.sha256.len() != 32
                || receipt.sha256.iter().all(|byte| *byte == 0)
                || receipt.custody_transfer_source_proof.is_empty()
                || receipt.custody_transfer_source_proof.len()
                    > CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1
        })
    {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(operation_id)
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    label: &[u8],
    command_message_id: [u8; 16],
    operation_id: [u8; 16],
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    build(
        digest16(label, &operation_id, &command_message_id),
        operation_id,
        command_message_id.to_vec(),
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: operation_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

fn build(
    message_id: [u8; 16],
    partition_key: [u8; 16],
    causation_message_id: Vec<u8>,
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: CONTACTS_MAIL_SYNC_SOURCE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_MAJOR_V1,
            revision: CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"contacts-mail-sync-source-runtime-v1",
                context.runtime_instance_id.as_bytes(),
                context.module_id.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: partition_key.to_vec(),
        causation_message_id,
        correlation_id: partition_key.to_vec(),
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
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &ContactsMailSyncSourceEnvelopeContextV1,
) -> Result<(), ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    if !valid_owner(&context.module_id)
        || !valid_bounded(&context.runtime_instance_id, 128)
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn timestamp(context: &ContactsMailSyncSourceEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ContactsMailSyncSourceEnvelopeBuildErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload)?;
    if value.iter().all(|byte| *byte == 0) {
        return Err(ContactsMailSyncSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn valid_owner(value: &str) -> bool {
    valid_bounded(value, 128)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_bounded(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    digest32(label, left, right)[..16]
        .try_into()
        .expect("fixed digest")
}

fn digest32(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(label);
    hash.update([0]);
    hash.update(left);
    hash.update([0]);
    hash.update(right);
    hash.finalize().into()
}

fn outbox_error(_: OutboxRecordError) -> ContactsMailSyncSourceEnvelopeBuildErrorV1 {
    ContactsMailSyncSourceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{ContactMailSyncSourceContentReceiptV1, ContactMailSyncSourceRejectCodeV1};
    use makosh_events_protocol::{
        v1::durable_envelope_v1::Semantics, validation::envelope::decode_envelope_v1,
    };

    #[test]
    fn changed_event_is_revision_bound_and_private_field_free() {
        let record = build_contact_changed_for_mail_sync_outbox_record_v1(
            ContactChangedForMailSyncV1 {
                contact_id: vec![1; 16],
                contact_revision: 7,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-contacts-runtime"),
        )
        .expect("event");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert!(matches!(envelope.semantics, Some(Semantics::Event(_))));
        assert_eq!(envelope.partition_key, vec![1; 16]);
        assert!(!envelope.payload.windows(5).any(|part| part == b"gmail"));
    }

    #[test]
    fn prepare_and_terminal_results_preserve_exact_operation_causation() {
        let operation_id = [2; 16];
        let prepare = build_contact_mail_sync_source_prepare_outbox_record_v1(
            PrepareContactMailSyncSourceCommandV1 {
                operation_id: operation_id.to_vec(),
                contact_id: vec![3; 16],
                expected_contact_revision: 4,
                target_mail_account_id: "account-1".to_owned(),
                logical_owner_id: "owner-1".to_owned(),
            },
            1_800_000_100,
            &context("makosh-mail-contacts-sync-runtime"),
        )
        .expect("prepare");
        let prepared = build_contact_mail_sync_source_prepared_outbox_record_v1(
            *prepare.message_id(),
            ContactMailSyncSourcePreparedV1 {
                operation_id: operation_id.to_vec(),
                contact_id: vec![3; 16],
                contact_revision: 4,
                source_content: Some(ContactMailSyncSourceContentReceiptV1 {
                    reference_id: vec![5; 16],
                    declared_bytes: 100,
                    sha256: vec![6; 32],
                    custody_transfer_source_proof: vec![7; 32],
                }),
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-contacts-runtime"),
        )
        .expect("prepared");
        let envelope = decode_envelope_v1(prepared.exact_bytes()).expect("envelope");
        assert_eq!(envelope.causation_message_id, prepare.message_id());
        assert!(matches!(envelope.semantics, Some(Semantics::Result(_))));

        let rejected = build_contact_mail_sync_source_rejected_outbox_record_v1(
            *prepare.message_id(),
            ContactMailSyncSourceRejectedV1 {
                operation_id: operation_id.to_vec(),
                code: ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodePolicy
                    as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-contacts-runtime"),
        );
        assert!(rejected.is_ok());
    }

    fn context(module_id: &str) -> ContactsMailSyncSourceEnvelopeContextV1 {
        ContactsMailSyncSourceEnvelopeContextV1 {
            module_id: module_id.to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }
}
