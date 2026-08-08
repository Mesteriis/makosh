use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const ATTACHMENT_TEXT_EXTRACTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_text_extraction.sql");
pub const ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_translation_source.sql");

#[must_use]
pub fn attachment_text_extraction_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "attachment_text_extraction".to_owned(),
        owner_id: "attachment_text_extraction".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "attachment_text_extraction_initial".to_owned(),
                forward_sql_utf8: ATTACHMENT_TEXT_EXTRACTION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_TEXT_EXTRACTION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "attachment_text_extraction_translation_source".to_owned(),
                forward_sql_utf8: ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_SCHEMA_V1)
                    .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_contains_only_owner_local_metadata_and_no_plaintext() {
        validate_storage_bundle(&attachment_text_extraction_storage_bundle_v1())
            .expect("valid text extraction bundle");
        let sql = attachment_text_extraction_storage_bundle_v1()
            .steps
            .into_iter()
            .map(|step| String::from_utf8(step.forward_sql_utf8).expect("utf8"))
            .collect::<Vec<_>>()
            .join("\n");
        for required in [
            "attachment_text_extraction_runs",
            "attachment_text_extraction_event_inbox",
            "attachment_text_extraction_scan_candidates",
            "attachment_text_extraction_safety_facts",
            "attachment_text_extraction_custody_outbox",
            "attachment_text_extraction_custody_result_inbox",
            "attachment_text_extraction_jobs",
            "attachment_text_extraction_artifacts",
            "attachment_text_extraction_realtime",
            "attachment_text_extraction_translation_source_inbox",
            "attachment_text_extraction_translation_source_outbox",
            "runtime_generation",
            "grant_epoch",
            "lease_fence",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "text_utf8",
            "extracted_content",
            "source_bytes",
            "provider_id",
            "filename",
            "mime_type",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
