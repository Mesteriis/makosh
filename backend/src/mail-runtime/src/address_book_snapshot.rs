//! Target-bound Contacts snapshot custody and decoding for Mail.

use std::{os::unix::net::UnixStream, time::Duration};

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_contacts_mail_sync_source_api::{
    CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1, CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1,
    wire::ContactMailSyncSourceContentV1,
};
use makosh_mail_address_book_persistence::{
    MailAddressBookTargetSnapshotReceiptV1, PendingMailAddressBookUpsertV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::BlobDataOperationV1,
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const MAX_CONTACT_VALUES: usize = 64;
const MAX_CONTACT_TEXT_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookSnapshotErrorV1 {
    InvalidReceipt,
    CustodyDenied,
    Unavailable,
}

pub fn transfer_contact_snapshot_custody_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    job: &PendingMailAddressBookUpsertV1,
) -> Result<MailAddressBookTargetSnapshotReceiptV1, MailAddressBookSnapshotErrorV1> {
    let admission = &job.admission;
    if !(1..=CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1)
        .contains(&admission.contact_snapshot_declared_bytes)
        || job.target_snapshot_receipt.is_some()
    {
        return Err(MailAddressBookSnapshotErrorV1::InvalidReceipt);
    }
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let transfer = request_managed_blob_custody_transfer_v2(
            channel,
            &mut dispatcher,
            ManagedBlobCustodyTransferRequestV1 {
                capability_id: CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
                source_reference_id: &admission.contact_snapshot_reference_id,
                declared_size: admission.contact_snapshot_declared_bytes,
                receipt_sha256: &admission.contact_snapshot_sha256,
                custody_source_proof: &admission.contact_snapshot_custody_source_proof,
                evidence_id: &admission.command_message_id,
                evidence_envelope_sha256: &admission.command_envelope_sha256,
            },
        )
        .map_err(|_| MailAddressBookSnapshotErrorV1::CustodyDenied)?;
        let target_reference_id: [u8; 16] = transfer
            .grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| MailAddressBookSnapshotErrorV1::InvalidReceipt)?;
        BlobDataClient::new(transfer.data_socket_path)
            .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
            .map_err(|_| MailAddressBookSnapshotErrorV1::Unavailable)?;
        Ok(MailAddressBookTargetSnapshotReceiptV1 {
            reference_id: target_reference_id,
            receipt_sha256: admission.contact_snapshot_sha256,
        })
    })
}

pub fn read_contact_snapshot_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    job: &PendingMailAddressBookUpsertV1,
    receipt: &MailAddressBookTargetSnapshotReceiptV1,
) -> Result<ContactMailSyncSourceContentV1, MailAddressBookSnapshotErrorV1> {
    let admission = &job.admission;
    if !(1..=CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1)
        .contains(&admission.contact_snapshot_declared_bytes)
        || receipt.receipt_sha256 != admission.contact_snapshot_sha256
        || receipt.reference_id.iter().all(|byte| *byte == 0)
    {
        return Err(MailAddressBookSnapshotErrorV1::InvalidReceipt);
    }
    blocking(channel, |channel| {
        let mut dispatcher = RejectManagedControlRequestsV2;
        let session = request_managed_blob_session_v2(
            channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                reference_id: &receipt.reference_id,
                declared_size: admission.contact_snapshot_declared_bytes,
                backup_class: 1,
                receipt_sha256: Some(&receipt.receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| MailAddressBookSnapshotErrorV1::CustodyDenied)?;
        let bytes = Zeroizing::new(
            BlobDataClient::new(session.data_socket_path)
                .and_then(|client| {
                    client.read_range(
                        session.grant,
                        session.channel_binding,
                        0,
                        admission.contact_snapshot_declared_bytes,
                    )
                })
                .map_err(|_| MailAddressBookSnapshotErrorV1::Unavailable)?,
        );
        if bytes.len()
            != usize::try_from(admission.contact_snapshot_declared_bytes).unwrap_or(usize::MAX)
            || Sha256::digest(bytes.as_slice()).as_slice() != admission.contact_snapshot_sha256
        {
            return Err(MailAddressBookSnapshotErrorV1::InvalidReceipt);
        }
        let source = ContactMailSyncSourceContentV1::decode(bytes.as_slice())
            .map_err(|_| MailAddressBookSnapshotErrorV1::InvalidReceipt)?;
        validate_source(&source)?;
        Ok(source)
    })
}

fn validate_source(
    source: &ContactMailSyncSourceContentV1,
) -> Result<(), MailAddressBookSnapshotErrorV1> {
    if !valid_text(&source.display_name)
        || source.email_addresses.len() > MAX_CONTACT_VALUES
        || source.phone_numbers.len() > MAX_CONTACT_VALUES
        || source.email_addresses.is_empty() && source.phone_numbers.is_empty()
        || source
            .email_addresses
            .iter()
            .any(|value| !valid_text(value))
        || source.phone_numbers.iter().any(|value| !valid_text(value))
        || source.target_account_link.as_ref().is_some_and(|link| {
            !valid_ascii(&link.provider_entry_id, 512)
                || link
                    .provider_etag
                    .as_deref()
                    .is_some_and(|value| !valid_ascii(value, 512))
        })
    {
        return Err(MailAddressBookSnapshotErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CONTACT_TEXT_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_ascii(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value.is_ascii()
        && value.trim() == value
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn blocking<T>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    operation: impl FnOnce(
        &mut ManagedControlChannelV2<UnixStream>,
    ) -> Result<T, MailAddressBookSnapshotErrorV1>,
) -> Result<T, MailAddressBookSnapshotErrorV1> {
    channel
        .inner_mut()
        .set_nonblocking(false)
        .and_then(|_| {
            channel
                .inner_mut()
                .set_read_timeout(Some(Duration::from_secs(5)))
        })
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(Duration::from_secs(5)))
        })
        .map_err(|_| MailAddressBookSnapshotErrorV1::Unavailable)?;
    let result = operation(channel);
    let restored = channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .and_then(|_| channel.inner_mut().set_nonblocking(true))
        .map_err(|_| MailAddressBookSnapshotErrorV1::Unavailable);
    match (result, restored) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use makosh_contacts_mail_sync_source_api::wire::MailAddressBookLinkV1;

    use super::*;

    #[test]
    fn private_snapshot_validation_is_bounded_and_link_exact() {
        let valid = ContactMailSyncSourceContentV1 {
            display_name: "Ada Lovelace".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
            target_account_link: Some(MailAddressBookLinkV1 {
                provider_entry_id: "people/ada".to_owned(),
                provider_etag: Some("etag-1".to_owned()),
            }),
        };
        assert_eq!(validate_source(&valid), Ok(()));

        let mut missing_identity = valid.clone();
        missing_identity.email_addresses.clear();
        assert_eq!(
            validate_source(&missing_identity),
            Err(MailAddressBookSnapshotErrorV1::InvalidReceipt)
        );

        let mut invalid_link = valid;
        invalid_link
            .target_account_link
            .as_mut()
            .expect("link")
            .provider_entry_id = "people/\nprivate".to_owned();
        assert_eq!(
            validate_source(&invalid_link),
            Err(MailAddressBookSnapshotErrorV1::InvalidReceipt)
        );
    }
}
