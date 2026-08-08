use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const ATTACHMENT_TRANSLATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_translation.sql");
pub const ATTACHMENT_TRANSLATION_READ_TICKETS_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_translation_read_tickets.sql");

#[must_use]
pub fn attachment_translation_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "attachment_translation".to_owned(),
        owner_id: "attachment_translation".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "attachment_translation_initial".to_owned(),
                forward_sql_utf8: ATTACHMENT_TRANSLATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_TRANSLATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "attachment_translation_read_tickets".to_owned(),
                forward_sql_utf8: ATTACHMENT_TRANSLATION_READ_TICKETS_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_TRANSLATION_READ_TICKETS_SCHEMA_V1).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_durable_and_private_source_negative() {
        let bundle = attachment_translation_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid translation storage bundle");
        assert_eq!(bundle.owner_id, "attachment_translation");
        let sql = std::str::from_utf8(ATTACHMENT_TRANSLATION_SCHEMA_V1).expect("utf8");
        for required in [
            "attachment_translation_runs",
            "request_fingerprint",
            "source_extraction_run_id",
            "inference_request_digest",
            "pending_translated_sha256",
            "artifact_translated_sha256",
            "attachment_translation_inbox",
            "attachment_translation_outbox",
            "attachment_translation_realtime",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "communications_",
            "mail_",
            "telegram_",
            "whatsapp_",
            "zulip_",
            "source_body",
            "prompt",
            "provider_id",
            "model_id",
            "endpoint",
            "translated_text",
            "source_text",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
