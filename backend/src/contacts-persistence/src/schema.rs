use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const CONTACTS_STORAGE_BUNDLE_REVISION_V1: u32 = 3;
pub const CONTACTS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_contacts.sql");
pub const CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_mail_sync_source.sql");
pub const CONTACTS_MAIL_PROVIDER_LINK_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_mail_provider_link_command.sql");

#[must_use]
pub fn contacts_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: CONTACTS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "contacts".to_owned(),
        owner_id: "contacts".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "contacts_initial".to_owned(),
                forward_sql_utf8: CONTACTS_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(CONTACTS_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "contacts_mail_sync_source".to_owned(),
                forward_sql_utf8: CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: CONTACTS_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "contacts_mail_provider_link_command".to_owned(),
                forward_sql_utf8: CONTACTS_MAIL_PROVIDER_LINK_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(CONTACTS_MAIL_PROVIDER_LINK_SCHEMA_V3).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_contacts_owned_and_has_atomic_boundaries() {
        let bundle = contacts_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Contacts storage bundle");
        assert_eq!(bundle.owner_id, "contacts");
        assert_eq!(bundle.steps.len(), 3);
        let sql = [
            CONTACTS_SCHEMA_V1,
            CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_V2,
            CONTACTS_MAIL_PROVIDER_LINK_SCHEMA_V3,
        ]
        .concat();
        let sql = std::str::from_utf8(&sql).expect("utf8");
        for required in [
            "contacts_mail_entry_inbox",
            "contacts_state",
            "contacts_email_identities",
            "contacts_phone_identities",
            "contacts_provider_links",
            "contacts_outbox",
            "contacts_mail_sync_source_inbox",
            "contacts_mail_provider_link_inbox",
            "command_envelope_sha256",
            "reject_code",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "mail_credential",
            "communications_",
            "tasks_",
            "review_",
            "calendar_",
            "provider_payload",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
