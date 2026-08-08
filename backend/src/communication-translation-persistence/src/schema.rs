use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_TRANSLATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_translation.sql");

#[must_use]
pub fn communication_translation_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communication_translation".to_owned(),
        owner_id: "communication_translation".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "communication_translation_initial".to_owned(),
            forward_sql_utf8: COMMUNICATION_TRANSLATION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(COMMUNICATION_TRANSLATION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_durable_and_private_source_negative() {
        let bundle = communication_translation_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid translation storage bundle");
        assert_eq!(bundle.owner_id, "communication_translation");
        let sql = std::str::from_utf8(COMMUNICATION_TRANSLATION_SCHEMA_V1).expect("utf8");
        for required in [
            "communication_translation_runs",
            "request_fingerprint",
            "source_evidence_id",
            "inference_request_digest",
            "candidate_translated_text_utf8",
            "candidate_detected_source_language",
            "candidate_target_language",
            "communication_translation_inbox",
            "communication_translation_outbox",
            "communication_translation_realtime",
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
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
