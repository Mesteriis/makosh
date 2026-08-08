use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const WHISPER_STT_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const WHISPER_STT_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_whisper_stt_runs.sql");

#[must_use]
pub fn whisper_stt_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: WHISPER_STT_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "whisper_stt_runs".to_owned(),
        owner_id: "whisper_stt".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "whisper_stt_runs_initial".to_owned(),
            forward_sql_utf8: WHISPER_STT_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(WHISPER_STT_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_is_owner_local_and_private_content_negative() {
        let bundle = whisper_stt_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("storage bundle");
        assert_eq!(bundle.owner_id, "whisper_stt");
        let sql = std::str::from_utf8(WHISPER_STT_SCHEMA_V1).expect("schema utf8");
        for required in [
            "request_digest",
            "source_sha256",
            "model_revision_sha256",
            "transcript_sha256",
            "run_state BETWEEN 1 AND 5",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "audio_bytes",
            "transcript_text",
            "segment_text",
            "custody_proof",
            "filesystem_path",
            "stdout",
            "stderr",
            "communications_",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
