use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_attachment_preview_evidence_replay.sql");
pub const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_provider_neutral_anchor_replay.sql");

#[must_use]
pub fn attachment_preview_evidence_replay_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "attachment_preview_evidence_replay_state".to_owned(),
        owner_id: "attachment_preview_evidence_replay".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "attachment_preview_evidence_replay_initial".to_owned(),
                forward_sql_utf8: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "attachment_preview_evidence_replay_provider_neutral_anchor"
                    .to_owned(),
                forward_sql_utf8: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V2).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_is_owner_local_metadata_only_and_has_exact_inbox_outbox() {
        validate_storage_bundle(&attachment_preview_evidence_replay_storage_bundle_v1())
            .expect("bundle");
        let sql = std::str::from_utf8(ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V2).expect("utf8");
        for required in [
            "attachment_preview_evidence_replay_anchor_producers",
            "attachment_preview_evidence_replay_anchor_result_messages",
            "attachment_preview_evidence_replay_anchor_command_outbox",
            "attachment_preview_evidence_replay_anchor_result_inbox",
            "exact_envelope_bytes",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "communications_domain_outbox",
            "mail_attachment_security_outbox",
            "payload_bytes",
            "subject",
            "blob",
            "provider_content",
            "producer_registration_id",
            "producer_runtime_generation",
            "producer_grant_epoch",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
