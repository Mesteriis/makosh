use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_archive_inspection.sql");

#[must_use]
pub fn attachment_archive_inspection_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "attachment_archive_inspection".to_owned(),
        owner_id: "attachment_archive_inspection".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "attachment_archive_inspection_initial".to_owned(),
            forward_sql_utf8: ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_engine_owned_and_contains_only_private_state() {
        let bundle = attachment_archive_inspection_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid archive inspection bundle");
        assert_eq!(bundle.owner_id, "attachment_archive_inspection");
        let sql = std::str::from_utf8(ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_V1).expect("utf8");
        for required in [
            "attachment_archive_inspection_runs",
            "attachment_archive_inspection_event_inbox",
            "attachment_archive_inspection_scan_candidates",
            "attachment_archive_inspection_safety_facts",
            "attachment_archive_inspection_jobs",
            "attachment_archive_inspection_reports",
            "attachment_archive_inspection_report_entries",
            "attachment_archive_inspection_realtime",
            "lease_fence",
            "runtime_generation",
            "grant_epoch",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "mail_",
            "telegram_",
            "whatsapp_",
            "zulip_",
            "provider_id",
            "provider_path",
            "message_body",
            "archive_bytes",
            "extracted_content",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
