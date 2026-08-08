use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const ATTACHMENT_PREVIEW_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_attachment_preview.sql");

#[must_use]
pub fn attachment_preview_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "attachment_preview".to_owned(),
        owner_id: "attachment_preview".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "attachment_preview_initial".to_owned(),
            forward_sql_utf8: ATTACHMENT_PREVIEW_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(ATTACHMENT_PREVIEW_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_is_owner_local_and_contains_no_private_content_or_ticket_plaintext() {
        validate_storage_bundle(&attachment_preview_storage_bundle_v1())
            .expect("valid preview bundle");
        let sql = std::str::from_utf8(ATTACHMENT_PREVIEW_SCHEMA_V1).expect("utf8");
        for required in [
            "attachment_preview_runs",
            "attachment_preview_event_inbox",
            "attachment_preview_scan_candidates",
            "attachment_preview_safety_facts",
            "attachment_preview_custody_outbox",
            "attachment_preview_custody_result_inbox",
            "attachment_preview_jobs",
            "attachment_preview_artifacts",
            "attachment_preview_read_tickets",
            "attachment_preview_realtime",
            "runtime_generation",
            "grant_epoch",
            "lease_fence",
            "ticket_sha256",
            "device_actor_sha256",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "ticket_plaintext",
            "source_bytes",
            "preview_bytes",
            "text_utf8",
            "provider_id",
            "account_id",
            "filename",
            "mime_type",
            "filesystem_path",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
