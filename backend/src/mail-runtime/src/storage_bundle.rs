//! Exact Mail storage successor composed by the managed runtime.

use makosh_mail_address_book_persistence::{
    MailAddressBookSchemaErrorV1, append_mail_address_book_storage_v1,
};
use makosh_mail_persistence::{
    MailIcloudCardDavCredentialSchemaErrorV1, MailSyncDeadlineFailureSchemaErrorV1,
    append_mail_icloud_carddav_credential_storage_v1, append_mail_sync_deadline_failure_storage_v1,
    mail_storage_bundle_v1,
};
use makosh_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplayDeliverySchemaErrorV1, MailRetainedEvidenceReplayScanSchemaErrorV1,
    MailRetainedEvidenceReplaySchemaErrorV1,
    append_mail_retained_evidence_replay_delivery_storage_v1,
    append_mail_retained_evidence_replay_scan_storage_v1,
    append_mail_retained_evidence_replay_storage_v1,
};
use makosh_storage_protocol::v1::StorageBundleV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRuntimeStorageBundleErrorV1 {
    RetainedEvidenceReplay(MailRetainedEvidenceReplaySchemaErrorV1),
    RetainedEvidenceReplayDelivery(MailRetainedEvidenceReplayDeliverySchemaErrorV1),
    RetainedEvidenceReplayScan(MailRetainedEvidenceReplayScanSchemaErrorV1),
    AddressBook(MailAddressBookSchemaErrorV1),
    IcloudCardDavCredential(MailIcloudCardDavCredentialSchemaErrorV1),
    SyncDeadlineFailure(MailSyncDeadlineFailureSchemaErrorV1),
}

pub fn mail_runtime_storage_bundle_v1() -> Result<StorageBundleV1, MailRuntimeStorageBundleErrorV1>
{
    let bundle = append_mail_retained_evidence_replay_storage_v1(mail_storage_bundle_v1())
        .map_err(MailRuntimeStorageBundleErrorV1::RetainedEvidenceReplay)?;
    let bundle = append_mail_retained_evidence_replay_delivery_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::RetainedEvidenceReplayDelivery)?;
    let bundle = append_mail_retained_evidence_replay_scan_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::RetainedEvidenceReplayScan)?;
    let bundle = append_mail_address_book_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::AddressBook)?;
    let bundle = append_mail_icloud_carddav_credential_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::IcloudCardDavCredential)?;
    append_mail_sync_deadline_failure_storage_v1(bundle)
        .map_err(MailRuntimeStorageBundleErrorV1::SyncDeadlineFailure)
}

#[cfg(test)]
mod tests {
    use super::mail_runtime_storage_bundle_v1;

    #[test]
    fn storage_bundle_has_exact_successor_lineage() {
        let bundle = mail_runtime_storage_bundle_v1().expect("Mail storage bundle");
        assert_eq!(bundle.revision, 31);
        assert_eq!(bundle.steps.last().map(|step| step.revision), Some(31));
    }
}
