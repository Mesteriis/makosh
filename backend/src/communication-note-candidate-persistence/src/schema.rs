use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_NOTE_CANDIDATE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_note_candidate.sql");

#[must_use]
pub fn communication_note_candidate_extraction_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "communication_note_candidate_extraction".to_owned(),
        owner_id: "communication_note_candidate_extraction".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: COMMUNICATION_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "communication_note_candidate_extraction_initial".to_owned(),
            forward_sql_utf8: COMMUNICATION_NOTE_CANDIDATE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(COMMUNICATION_NOTE_CANDIDATE_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_durable_and_private_source_negative() {
        let bundle = communication_note_candidate_extraction_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid note candidate storage bundle");
        assert_eq!(bundle.owner_id, "communication_note_candidate_extraction");
        let sql = std::str::from_utf8(COMMUNICATION_NOTE_CANDIDATE_SCHEMA_V1).expect("utf8");
        for required in [
            "communication_note_candidate_extraction_runs",
            "request_fingerprint",
            "source_evidence_id",
            "source_sha256",
            "candidate_bytes",
            "communication_note_candidate_extraction_inbox",
            "communication_note_candidate_extraction_outbox",
            "communication_note_candidate_extraction_realtime",
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
