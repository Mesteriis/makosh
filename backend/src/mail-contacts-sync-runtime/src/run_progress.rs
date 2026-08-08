use makosh_mail_address_book_contract::{
    MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, MailAddressBookEnvelopeContextV1,
    build_fetch_mail_address_book_page_command_v1, wire::FetchMailAddressBookPageCommandV1,
};
use makosh_mail_contacts_sync_persistence::{
    AdvanceMailContactsSyncPageV1, MailContactsSyncAdvanceOutcomeV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1, OutboxEnvelopeV1,
};
use sha2::{Digest, Sha256};

use crate::{
    MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1, MailContactsSyncProviderRuntimeContextV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncProgressErrorV1 {
    InvalidContext,
    Persistence(MailContactsSyncPersistenceErrorV1),
}

pub async fn advance_ready_page_v1(
    persistence: &MailContactsSyncPersistenceV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
    run_id: [u8; 16],
) -> Result<MailContactsSyncAdvanceOutcomeV1, MailContactsSyncProgressErrorV1> {
    if runtime.now_unix_millis <= 0
        || runtime.runtime_generation == 0
        || runtime.runtime_instance_id.is_empty()
    {
        return Err(MailContactsSyncProgressErrorV1::InvalidContext);
    }
    let run = persistence
        .load_run(&runtime.logical_owner_id, &run_id)
        .await
        .map_err(MailContactsSyncProgressErrorV1::Persistence)?;
    let next_page_command = if let Some(cursor) = run.status.continuation_cursor.clone() {
        let next_page_sequence = run.status.page_sequence.checked_add(1).ok_or(
            MailContactsSyncProgressErrorV1::Persistence(
                MailContactsSyncPersistenceErrorV1::InvalidInput,
            ),
        )?;
        let command_id = digest16(
            b"mail-contacts-sync/fetch-page/v1",
            &run_id,
            &next_page_sequence.to_be_bytes(),
        );
        let record = build_fetch_mail_address_book_page_command_v1(
            FetchMailAddressBookPageCommandV1 {
                command_id: command_id.to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: runtime.logical_owner_id.clone(),
                account_id: run.draft.account_id,
                page_sequence: next_page_sequence,
                continuation_cursor: Some(cursor),
                page_size: MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1,
            },
            runtime.now_unix_millis / 1_000 + MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
            &MailAddressBookEnvelopeContextV1 {
                module_id: makosh_mail_contacts_sync_api::MAIL_CONTACTS_SYNC_MODULE_ID_V1
                    .to_owned(),
                runtime_instance_id: runtime.runtime_instance_id.clone(),
                runtime_generation: runtime.runtime_generation,
                recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
                recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
                    .unwrap_or_default(),
            },
        )
        .map_err(|_| MailContactsSyncProgressErrorV1::InvalidContext)?;
        Some(OutboxEnvelopeV1 {
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            envelope_bytes: record.exact_bytes().to_vec(),
        })
    } else {
        None
    };
    persistence
        .advance_ready_page(&AdvanceMailContactsSyncPageV1 {
            logical_owner_id: runtime.logical_owner_id.clone(),
            run_id,
            next_page_command,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncProgressErrorV1::Persistence)
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update((left.len() as u64).to_be_bytes());
    digest.update(left);
    digest.update((right.len() as u64).to_be_bytes());
    digest.update(right);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_page_command_identity_is_stable_per_page() {
        assert_eq!(
            digest16(b"domain", &[1; 16], &2_u64.to_be_bytes()),
            digest16(b"domain", &[1; 16], &2_u64.to_be_bytes())
        );
        assert_ne!(
            digest16(b"domain", &[1; 16], &2_u64.to_be_bytes()),
            digest16(b"domain", &[1; 16], &3_u64.to_be_bytes())
        );
    }
}
