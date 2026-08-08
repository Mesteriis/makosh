use makosh_storage_protocol::{
    v1::{StorageBundleV1, StorageMigrationStepV1},
    validation::validate_storage_bundle,
};
use sha2::{Digest, Sha256};

const MAIL_ADDRESS_BOOK_PREDECESSOR_REVISION_V1: u32 = 25;
pub const MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1: u32 = 28;
pub const MAIL_ADDRESS_BOOK_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_address_book_upsert.sql");
pub const MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_snapshot_custody.sql");
pub const MAIL_ADDRESS_BOOK_PROVIDER_PAGE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_provider_page.sql");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

pub fn append_mail_address_book_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailAddressBookSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != MAIL_ADDRESS_BOOK_PREDECESSOR_REVISION_V1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailAddressBookSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: 26,
        migration_id: "mail_address_book_upsert".to_owned(),
        forward_sql_utf8: MAIL_ADDRESS_BOOK_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_ADDRESS_BOOK_SCHEMA_V1).to_vec(),
    });
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: 27,
        migration_id: "mail_address_book_snapshot_custody".to_owned(),
        forward_sql_utf8: MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1).to_vec(),
    });
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_address_book_provider_page".to_owned(),
        forward_sql_utf8: MAIL_ADDRESS_BOOK_PROVIDER_PAGE_SCHEMA_V1.to_vec(),
        sha256: Sha256::digest(MAIL_ADDRESS_BOOK_PROVIDER_PAGE_SCHEMA_V1).to_vec(),
    });
    predecessor.revision = MAIL_ADDRESS_BOOK_STORAGE_BUNDLE_REVISION_V1;
    validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailAddressBookSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::v1::StorageMigrationStepV1;

    use super::*;

    fn predecessor(owner_id: &str) -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 25,
            bundle_id: "mail_state".to_owned(),
            owner_id: owner_id.to_owned(),
            steps: (1..=23)
                .map(|revision| {
                    let sql = format!("CREATE TABLE makosh_data.mail_test_{revision} (id BIGINT);")
                        .into_bytes();
                    StorageMigrationStepV1 {
                        revision: if revision <= 20 {
                            revision
                        } else {
                            revision + 2
                        },
                        migration_id: format!("mail_test_{revision}"),
                        sha256: Sha256::digest(&sql).to_vec(),
                        forward_sql_utf8: sql,
                    }
                })
                .collect(),
        }
    }

    #[test]
    fn appends_only_mail_address_book_upsert_state() {
        let bundle =
            append_mail_address_book_storage_v1(predecessor("mail")).expect("valid successor");
        assert_eq!(bundle.revision, 28);
        let upsert_sql = String::from_utf8(
            bundle.steps[bundle.steps.len() - 3]
                .forward_sql_utf8
                .clone(),
        )
        .expect("upsert utf8");
        let custody_sql = String::from_utf8(
            bundle.steps[bundle.steps.len() - 2]
                .forward_sql_utf8
                .clone(),
        )
        .expect("custody utf8");
        let provider_page_sql =
            String::from_utf8(bundle.steps.last().unwrap().forward_sql_utf8.clone())
                .expect("provider page utf8");
        assert!(upsert_sql.contains("mail_address_book_upsert_inbox"));
        assert!(upsert_sql.contains("mail_address_book_upsert_result_outbox"));
        assert!(upsert_sql.contains("exact_envelope_bytes"));
        assert!(custody_sql.contains("target_contact_snapshot_reference_id"));
        assert!(custody_sql.contains("mail_address_book_target_snapshot_receipt_complete"));
        assert!(provider_page_sql.contains("mail_address_book_fetch_inbox"));
        assert!(provider_page_sql.contains("mail_address_book_fetch_outbox"));
        assert!(
            !format!("{upsert_sql}\n{custody_sql}").contains("CREATE TABLE makosh_data.contacts_")
        );
        assert!(!format!("{upsert_sql}\n{custody_sql}").contains("mail_contacts_sync_"));
    }

    #[test]
    fn rejects_wrong_owner_predecessor() {
        assert_eq!(
            append_mail_address_book_storage_v1(predecessor("contacts")),
            Err(MailAddressBookSchemaErrorV1::InvalidPredecessor),
        );
    }
}
