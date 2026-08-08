use makosh_mail_address_book_contract::{
    MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, MailAddressBookEnvelopeContextV1,
    build_fetch_mail_address_book_page_command_v1, wire::FetchMailAddressBookPageCommandV1,
};
use makosh_mail_contacts_sync_api::MAIL_CONTACTS_SYNC_MODULE_ID_V1;
use makosh_mail_contacts_sync_persistence::OutboxEnvelopeV1;
use sha2::{Digest, Sha256};

use crate::MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InitialFetchCommandContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub authoritative_now_unix_millis: i64,
}

pub(crate) fn build_initial_fetch_command_v1(
    run_id: [u8; 16],
    account_id: &str,
    context: &InitialFetchCommandContextV1,
) -> Result<OutboxEnvelopeV1, &'static str> {
    if context.authoritative_now_unix_millis <= 0 {
        return Err("mail_contacts_sync_invalid_time");
    }
    let command_id = digest16(
        b"mail-contacts-sync-fetch-page-command-v1",
        &run_id,
        &1_u64.to_be_bytes(),
    );
    let record = build_fetch_mail_address_book_page_command_v1(
        FetchMailAddressBookPageCommandV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: context.logical_owner_id.clone(),
            account_id: account_id.to_owned(),
            page_sequence: 1,
            continuation_cursor: None,
            page_size: MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1,
        },
        context.authoritative_now_unix_millis / 1_000
            + MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
        &MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_CONTACTS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.authoritative_now_unix_millis / 1_000,
            recorded_at_nanos: i32::try_from(
                (context.authoritative_now_unix_millis % 1_000) * 1_000_000,
            )
            .map_err(|_| "mail_contacts_sync_invalid_time")?,
        },
    )
    .map_err(|_| "mail_contacts_sync_fetch_command_invalid")?;
    Ok(OutboxEnvelopeV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    })
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
