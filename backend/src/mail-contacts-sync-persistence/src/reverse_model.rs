use makosh_mail_contacts_sync_core::MailContactsSyncRejectCodeV1;

use crate::{MailContactsSyncPersistenceErrorV1, OutboxEnvelopeV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncReverseOperationSeedV1 {
    pub operation_id: [u8; 16],
    pub configuration_instance_id: String,
    pub account_id: String,
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub origin_run_id: Option<[u8; 16]>,
    pub source_prepare_command: OutboxEnvelopeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptContactChangedForMailSyncV1 {
    pub logical_owner_id: String,
    pub event_message_id: [u8; 16],
    pub event_envelope_sha256: [u8; 32],
    pub operations: Vec<MailContactsSyncReverseOperationSeedV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptContactChangedForMailSyncOutcomeV1 {
    Applied { operations: u16 },
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncReverseOperationV1 {
    pub operation_id: [u8; 16],
    pub configuration_instance_id: String,
    pub account_id: String,
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub state: u8,
    pub origin_run_id: Option<[u8; 16]>,
    pub mail_command_message_id: Option<[u8; 16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContactMailSyncSourceV1 {
    pub logical_owner_id: String,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub mail_command: Option<OutboxEnvelopeV1>,
    pub rejected: bool,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteContactMailSyncSourceOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteMailAddressBookUpsertV1 {
    pub logical_owner_id: String,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub mail_command_message_id: [u8; 16],
    pub outcome: MailContactsSyncProviderWriteOutcomeV1,
    pub contacts_link_command: Option<OutboxEnvelopeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteMailAddressBookUpsertOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteContactsProviderLinkV1 {
    pub logical_owner_id: String,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub contacts_command_message_id: [u8; 16],
    pub reject_code: Option<MailContactsSyncRejectCodeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteContactsProviderLinkOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncProviderWriteOutcomeV1 {
    Succeeded,
    Rejected(MailContactsSyncRejectCodeV1),
    OutcomeUnknown,
}

pub(crate) fn validate_changed_input(
    input: &AcceptContactChangedForMailSyncV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.event_message_id.iter().all(|byte| *byte == 0)
        || input.event_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operations.len() > 32
        || input.occurred_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    let mut operation_ids = std::collections::BTreeSet::new();
    let mut configurations = std::collections::BTreeSet::new();
    for operation in &input.operations {
        if operation.operation_id.iter().all(|byte| *byte == 0)
            || !crate::model::valid_identity(&operation.configuration_instance_id)
            || operation.account_id.trim().is_empty()
            || operation.account_id.len() > 256
            || operation.account_id.chars().any(char::is_control)
            || operation.contact_id.iter().all(|byte| *byte == 0)
            || operation.contact_revision == 0
            || operation
                .origin_run_id
                .is_some_and(|run_id| run_id.iter().all(|byte| *byte == 0))
            || !crate::model::valid_envelope(&operation.source_prepare_command)
            || operation.source_prepare_command.message_id != operation.operation_id
            || !operation_ids.insert(operation.operation_id)
            || !configurations.insert(&operation.configuration_instance_id)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
    }
    Ok(())
}

pub(crate) fn validate_contacts_link_completion(
    input: &CompleteContactsProviderLinkV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.result_message_id.iter().all(|byte| *byte == 0)
        || input.result_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operation_id.iter().all(|byte| *byte == 0)
        || input
            .contacts_command_message_id
            .iter()
            .all(|byte| *byte == 0)
        || input.occurred_at_unix_millis <= 0
        || matches!(
            input.reject_code,
            Some(MailContactsSyncRejectCodeV1::OutcomeUnknown)
        )
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_source_completion(
    input: &CompleteContactMailSyncSourceV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.result_message_id.iter().all(|byte| *byte == 0)
        || input.result_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operation_id.iter().all(|byte| *byte == 0)
        || input.occurred_at_unix_millis <= 0
        || input.rejected == input.mail_command.is_some()
        || input.mail_command.as_ref().is_some_and(|command| {
            !crate::model::valid_envelope(command) || command.message_id == input.result_message_id
        })
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_mail_completion(
    input: &CompleteMailAddressBookUpsertV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !crate::model::valid_identity(&input.logical_owner_id)
        || input.result_message_id.iter().all(|byte| *byte == 0)
        || input.result_envelope_sha256.iter().all(|byte| *byte == 0)
        || input.operation_id.iter().all(|byte| *byte == 0)
        || input.mail_command_message_id.iter().all(|byte| *byte == 0)
        || input.occurred_at_unix_millis <= 0
        || matches!(
            input.outcome,
            MailContactsSyncProviderWriteOutcomeV1::Succeeded
        ) != input.contacts_link_command.is_some()
        || input.contacts_link_command.as_ref().is_some_and(|command| {
            !crate::model::valid_envelope(command) || command.message_id == input.result_message_id
        })
        || matches!(
            input.outcome,
            MailContactsSyncProviderWriteOutcomeV1::Rejected(
                MailContactsSyncRejectCodeV1::OutcomeUnknown
            )
        )
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_provider_outcome_has_one_explicit_representation() {
        let mut completion = CompleteMailAddressBookUpsertV1 {
            logical_owner_id: "owner-1".to_owned(),
            result_message_id: [1; 16],
            result_envelope_sha256: [2; 32],
            operation_id: [3; 16],
            mail_command_message_id: [4; 16],
            outcome: MailContactsSyncProviderWriteOutcomeV1::OutcomeUnknown,
            contacts_link_command: None,
            occurred_at_unix_millis: 1_800_000_000_000,
        };
        assert_eq!(validate_mail_completion(&completion), Ok(()));
        completion.outcome = MailContactsSyncProviderWriteOutcomeV1::Rejected(
            MailContactsSyncRejectCodeV1::OutcomeUnknown,
        );
        assert_eq!(
            validate_mail_completion(&completion),
            Err(MailContactsSyncPersistenceErrorV1::InvalidInput)
        );
    }
}
