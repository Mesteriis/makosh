use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const CALL_TRANSCRIPTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_call_transcription.sql");

#[must_use]
pub fn call_transcription_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "call_transcription".to_owned(),
        owner_id: "call_transcription".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "call_transcription_initial".to_owned(),
            forward_sql_utf8: CALL_TRANSCRIPTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(CALL_TRANSCRIPTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_durable_and_private_content_negative() {
        let bundle = call_transcription_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid call transcription storage bundle");
        assert_eq!(bundle.owner_id, "call_transcription");
        let sql = std::str::from_utf8(CALL_TRANSCRIPTION_SCHEMA_V1).expect("utf8");
        for required in [
            "call_transcription_runs",
            "call_transcription_inbox",
            "call_transcription_jobs",
            "call_transcription_outbox",
            "call_transcription_realtime",
            "call_transcription_read_tickets",
            "request_fingerprint",
            "source_receipt_sha256",
            "stt_result_receipt_sha256",
            "artifact_receipt_sha256",
            "client_session_sha256",
        ] {
            assert!(sql.contains(required), "missing {required}");
        }
        for forbidden in [
            "audio_bytes",
            "transcript_text",
            "segment_text",
            "custody_proof",
            "provider_id",
            "provider_name",
            "model_id",
            "model_name",
            "filesystem_path",
            "stdout",
            "stderr",
            "communications_",
            "telegram_",
        ] {
            assert!(!sql.contains(forbidden), "forbidden {forbidden}");
        }
    }
}
