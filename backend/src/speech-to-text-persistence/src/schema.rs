use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const SPEECH_TO_TEXT_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_speech_to_text.sql");
pub const SPEECH_TO_TEXT_OWNER_RLS_V2: &[u8] =
    include_bytes!("../migrations/0002_speech_to_text_owner_rls.sql");

#[must_use]
pub fn speech_to_text_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "speech_to_text".to_owned(),
        owner_id: "speech_to_text".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "speech_to_text_initial".to_owned(),
                forward_sql_utf8: SPEECH_TO_TEXT_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(SPEECH_TO_TEXT_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "speech_to_text_owner_rls".to_owned(),
                forward_sql_utf8: SPEECH_TO_TEXT_OWNER_RLS_V2.to_vec(),
                sha256: Sha256::digest(SPEECH_TO_TEXT_OWNER_RLS_V2).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_engine_owned_and_persists_no_private_content_or_custody_secret() {
        let bundle = speech_to_text_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid STT storage bundle");
        assert_eq!(bundle.owner_id, "speech_to_text");
        assert_eq!(bundle.revision, 2);
        assert_eq!(bundle.steps.len(), 2);
        let sql = std::str::from_utf8(SPEECH_TO_TEXT_SCHEMA_V1).expect("utf8");
        for required in [
            "speech_to_text_runs",
            "source_reference_id",
            "transcript_reference_id",
            "provider_settings_revision",
            "state_revision",
        ] {
            assert!(sql.contains(required), "missing {required}");
        }
        for forbidden in [
            "audio_bytes",
            "transcript_text",
            "segment_text",
            "custody_proof",
            "provider_name",
            "model_name",
            "filesystem_path",
            "stdout",
            "stderr",
        ] {
            assert!(!sql.contains(forbidden), "forbidden {forbidden}");
        }
        let rls = std::str::from_utf8(SPEECH_TO_TEXT_OWNER_RLS_V2).expect("RLS utf8");
        assert!(rls.contains("ENABLE ROW LEVEL SECURITY"));
        assert!(rls.contains("FORCE ROW LEVEL SECURITY"));
        assert_eq!(
            rls.matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            2
        );
    }
}
