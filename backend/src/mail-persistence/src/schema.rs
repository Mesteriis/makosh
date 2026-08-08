//! Immutable Mail-owned schema bundle for future independent Storage admission.

use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::{
    MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1, MAIL_SCHEMA_V1, MAIL_SCHEMA_V2, MAIL_SCHEMA_V3,
    MAIL_SCHEMA_V4, MAIL_SCHEMA_V5, MAIL_SCHEMA_V6, MAIL_SCHEMA_V7, MAIL_SCHEMA_V8, MAIL_SCHEMA_V9,
    MAIL_SCHEMA_V10, MAIL_SCHEMA_V11, MAIL_SCHEMA_V12, MAIL_SCHEMA_V13, MAIL_SCHEMA_V14,
    MAIL_SCHEMA_V15, MAIL_SCHEMA_V16, MAIL_SCHEMA_V17, MAIL_SCHEMA_V18, MAIL_SCHEMA_V19,
    MAIL_SCHEMA_V20,
};

pub const MAIL_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const MAIL_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const MAIL_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const MAIL_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const MAIL_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const MAIL_STORAGE_BUNDLE_REVISION_V6: u32 = 6;
pub const MAIL_STORAGE_BUNDLE_REVISION_V7: u32 = 7;
pub const MAIL_STORAGE_BUNDLE_REVISION_V8: u32 = 8;
pub const MAIL_STORAGE_BUNDLE_REVISION_V9: u32 = 9;
pub const MAIL_STORAGE_BUNDLE_REVISION_V10: u32 = 10;
pub const MAIL_STORAGE_BUNDLE_REVISION_V11: u32 = 11;
pub const MAIL_STORAGE_BUNDLE_REVISION_V12: u32 = 12;
pub const MAIL_STORAGE_BUNDLE_REVISION_V13: u32 = 13;
pub const MAIL_STORAGE_BUNDLE_REVISION_V14: u32 = 14;
pub const MAIL_STORAGE_BUNDLE_REVISION_V15: u32 = 15;
pub const MAIL_STORAGE_BUNDLE_REVISION_V16: u32 = 16;
pub const MAIL_STORAGE_BUNDLE_REVISION_V17: u32 = 17;
pub const MAIL_STORAGE_BUNDLE_REVISION_V18: u32 = 18;
pub const MAIL_STORAGE_BUNDLE_REVISION_V19: u32 = 19;
pub const MAIL_STORAGE_BUNDLE_REVISION_V20: u32 = 20;
/// Recovery successor for an admitted-but-never-applied development revision 21.
///
/// Migration steps remain the immutable admitted sequence 1..=20. Storage
/// bundle revisions fence release identity and do not require a same-numbered
/// migration step.
pub const MAIL_STORAGE_BUNDLE_REVISION_V22: u32 = 22;
pub const MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1: u32 = 29;
pub const MAIL_SYNC_DEADLINE_FAILURE_STORAGE_BUNDLE_REVISION_V1: u32 = 31;

pub const MAIL_SYNC_DEADLINE_FAILURE_SCHEMA_V1: &str = r#"
ALTER TABLE makosh_data.mail_sync_runs
    ADD COLUMN deadline_exceeded BOOLEAN NOT NULL DEFAULT FALSE;
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailIcloudCardDavCredentialSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncDeadlineFailureSchemaErrorV1 {
    InvalidPredecessor,
    InvalidSuccessor,
}

/// Returns the complete Mail schema as one immutable initial Storage bundle.
///
/// Mail remains an integration owner: this bundle has no Communications SQL,
/// foreign keys, or runtime dependency. Storage Control admits it separately
/// from the Communications first-owner inventory.
#[must_use]
pub fn mail_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_STORAGE_BUNDLE_REVISION_V22,
        bundle_id: "mail_state".to_owned(),
        owner_id: "mail".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "mail_state_initial".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "mail_attachment_security_outbox".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V2.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V2.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "mail_delivery_command_queue".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V3.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V3.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "mail_gmail_oauth_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V4.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V4.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "mail_outbound_attachment_manifest".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V5.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V5.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V6,
                migration_id: "mail_communications_outbox_causal_order".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V6.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V6.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V7,
                migration_id: "mail_account_credential_bindings".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V7.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V7.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V8,
                migration_id: "mail_account_lifecycle".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V8.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V8.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V9,
                migration_id: "mail_operational_projection".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V9.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V9.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V10,
                migration_id: "mail_sync_health".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V10.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V10.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V11,
                migration_id: "mail_composition".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V11.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V11.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V12,
                migration_id: "mail_message_flag_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V12.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V12.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V13,
                migration_id: "mail_stable_message_identity_and_imap_locator".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V13.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V13.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V14,
                migration_id: "mail_stable_message_identity_indexes".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V14.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V14.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V15,
                migration_id: "mail_message_location_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V15.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V15.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V16,
                migration_id: "mail_gmail_oauth_authority".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V16.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V16.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V17,
                migration_id: "mail_message_permanent_delete_operations".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V17.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V17.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V18,
                migration_id: "mail_delivery_route_locators".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V18.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V18.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V19,
                migration_id: "mail_delivery_intent_inbox_jobs_and_result_outbox".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V19.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V19.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_STORAGE_BUNDLE_REVISION_V20,
                migration_id: "mail_delivery_intent_custody_checkpoint".to_owned(),
                forward_sql_utf8: MAIL_SCHEMA_V20.as_bytes().to_vec(),
                sha256: Sha256::digest(MAIL_SCHEMA_V20.as_bytes()).to_vec(),
            },
        ],
    }
}

pub fn append_mail_icloud_carddav_credential_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailIcloudCardDavCredentialSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1 - 1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || makosh_storage_protocol::validation::validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailIcloudCardDavCredentialSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_icloud_carddav_credential_bindings".to_owned(),
        forward_sql_utf8: MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1.as_bytes()).to_vec(),
    });
    predecessor.revision = MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1;
    makosh_storage_protocol::validation::validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailIcloudCardDavCredentialSchemaErrorV1::InvalidSuccessor)
}

pub fn append_mail_sync_deadline_failure_storage_v1(
    mut predecessor: StorageBundleV1,
) -> Result<StorageBundleV1, MailSyncDeadlineFailureSchemaErrorV1> {
    if predecessor.major != 1
        || predecessor.revision != MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1
        || predecessor.bundle_id != "mail_state"
        || predecessor.owner_id != "mail"
        || predecessor.steps.last().map(|step| step.revision) != Some(predecessor.revision)
        || makosh_storage_protocol::validation::validate_storage_bundle(&predecessor).is_err()
    {
        return Err(MailSyncDeadlineFailureSchemaErrorV1::InvalidPredecessor);
    }
    predecessor.steps.push(StorageMigrationStepV1 {
        revision: MAIL_SYNC_DEADLINE_FAILURE_STORAGE_BUNDLE_REVISION_V1,
        migration_id: "mail_sync_deadline_failure_marker".to_owned(),
        forward_sql_utf8: MAIL_SYNC_DEADLINE_FAILURE_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(MAIL_SYNC_DEADLINE_FAILURE_SCHEMA_V1.as_bytes()).to_vec(),
    });
    predecessor.revision = MAIL_SYNC_DEADLINE_FAILURE_STORAGE_BUNDLE_REVISION_V1;
    makosh_storage_protocol::validation::validate_storage_bundle(&predecessor)
        .map(|()| predecessor)
        .map_err(|_| MailSyncDeadlineFailureSchemaErrorV1::InvalidSuccessor)
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owned_only_by_mail() {
        let bundle = mail_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "mail");
        assert_eq!(bundle.bundle_id, "mail_state");
        assert_eq!(bundle.revision, MAIL_STORAGE_BUNDLE_REVISION_V22);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(bundle.steps.len(), 20);
        let sql = bundle
            .steps
            .iter()
            .map(|step| {
                std::str::from_utf8(&step.forward_sql_utf8).expect("Mail Storage SQL is UTF-8")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 39);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS makosh_data.")
                .count(),
            39,
            "every Mail table belongs to the owner-scoped makosh_data schema"
        );
        assert!(sql.contains("mail_attachment_security_outbox"));
        assert!(sql.contains("mail_delivery_queue"));
        assert!(sql.contains("mail_gmail_oauth_attempts"));
        assert!(sql.contains("mail_gmail_oauth_operations"));
        assert!(sql.contains("mail_attachment_safety_projections"));
        assert!(sql.contains("mail_attachment_materializations"));
        assert!(sql.contains("mail_delivery_attachment_manifest"));
        assert!(sql.contains("causal_sequence"));
        assert!(sql.contains("mail_account_credential_bindings"));
        assert!(sql.contains("mail_account_lifecycle_operations"));
        assert!(sql.contains("mail_account_lifecycle_credentials"));
        assert!(sql.contains("mail_account_tombstones"));
        assert!(sql.contains("mail_operational_folders"));
        assert!(sql.contains("mail_operational_threads"));
        assert!(sql.contains("mail_operational_messages"));
        assert!(sql.contains("mail_operational_message_folders"));
        assert!(sql.contains("mail_delivery_route_accounts"));
        assert!(sql.contains("mail_delivery_route_conversations"));
        assert!(sql.contains("mail_delivery_route_messages"));
        assert!(sql.contains("mail_delivery_intent_inbox"));
        assert!(sql.contains("mail_delivery_intent_jobs"));
        assert!(sql.contains("mail_delivery_intent_result_outbox"));
        assert!(sql.contains("mail_delivery_intent_target_body_receipt_complete"));
        assert!(sql.contains("mail_sync_runs"));
        assert!(sql.contains("mail_sync_status"));
        assert!(sql.contains("mail_message_flag_operations"));
        assert!(sql.contains("mail_message_location_operations"));
        assert!(sql.contains("mail_message_permanent_delete_operations"));
        assert!(sql.contains("mail_imap_message_locators"));
        assert!(!sql.contains("makosh_data.attachment_security_"));
    }

    #[test]
    fn carddav_credential_successor_is_additive_and_mail_owned() {
        let mut predecessor = mail_storage_bundle_v1();
        predecessor.revision = 28;
        predecessor.steps.push(StorageMigrationStepV1 {
            revision: 28,
            migration_id: "mail_address_book_predecessor".to_owned(),
            forward_sql_utf8:
                b"CREATE TABLE makosh_data.mail_address_book_predecessor (id BIGINT);".to_vec(),
            sha256: Sha256::digest(
                b"CREATE TABLE makosh_data.mail_address_book_predecessor (id BIGINT);",
            )
            .to_vec(),
        });
        let bundle = append_mail_icloud_carddav_credential_storage_v1(predecessor)
            .expect("valid CardDAV credential successor");
        assert_eq!(bundle.revision, 29);
        let sql = std::str::from_utf8(&bundle.steps.last().unwrap().forward_sql_utf8)
            .expect("CardDAV credential SQL");
        assert!(sql.contains("mail_icloud_carddav_credential_bindings"));
        assert!(sql.contains("mail_icloud_carddav_lifecycle_credentials"));
        assert!(!sql.contains("DROP "));
        assert!(!sql.contains("communications"));
    }

    #[test]
    fn sync_deadline_failure_successor_adds_only_the_mail_marker() {
        let mut predecessor = mail_storage_bundle_v1();
        predecessor.revision = MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1;
        predecessor.steps.push(StorageMigrationStepV1 {
            revision: MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "mail_revision_29_predecessor".to_owned(),
            forward_sql_utf8: b"CREATE TABLE makosh_data.mail_revision_29_predecessor (id BIGINT);"
                .to_vec(),
            sha256: Sha256::digest(
                b"CREATE TABLE makosh_data.mail_revision_29_predecessor (id BIGINT);",
            )
            .to_vec(),
        });

        let bundle = append_mail_sync_deadline_failure_storage_v1(predecessor)
            .expect("valid sync deadline failure successor");
        assert_eq!(bundle.revision, 31);
        let sql = std::str::from_utf8(&bundle.steps.last().unwrap().forward_sql_utf8)
            .expect("sync deadline failure SQL");
        assert!(sql.contains("ADD COLUMN deadline_exceeded BOOLEAN NOT NULL DEFAULT FALSE"));
        assert!(!sql.contains("DROP "));
        assert!(!sql.contains("communications"));
    }
}
