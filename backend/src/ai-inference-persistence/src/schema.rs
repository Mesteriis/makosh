use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1: u32 = 6;
pub const AI_INFERENCE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_ai_inference_runs.sql");
pub const AI_SUMMARY_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0002_ai_summary_runs.sql");
pub const AI_TRANSLATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_ai_translation_runs.sql");
pub const AI_EXPLANATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0004_ai_explanation_runs.sql");
pub const AI_ATTACHMENT_TRANSLATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0005_ai_attachment_translation_runs.sql");
pub const AI_INFERENCE_OWNER_RLS_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0006_ai_inference_owner_rls.sql");

#[must_use]
pub fn ai_inference_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "ai_inference_runs".to_owned(),
        owner_id: "ai".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "ai_inference_runs_initial".to_owned(),
                forward_sql_utf8: AI_INFERENCE_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_INFERENCE_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "ai_summary_runs".to_owned(),
                forward_sql_utf8: AI_SUMMARY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_SUMMARY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 3,
                migration_id: "ai_translation_runs".to_owned(),
                forward_sql_utf8: AI_TRANSLATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_TRANSLATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 4,
                migration_id: "ai_explanation_runs".to_owned(),
                forward_sql_utf8: AI_EXPLANATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_EXPLANATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 5,
                migration_id: "ai_attachment_translation_runs".to_owned(),
                forward_sql_utf8: AI_ATTACHMENT_TRANSLATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_ATTACHMENT_TRANSLATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 6,
                migration_id: "ai_inference_owner_rls".to_owned(),
                forward_sql_utf8: AI_INFERENCE_OWNER_RLS_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(AI_INFERENCE_OWNER_RLS_SCHEMA_V1).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_ai_owned_typed_and_private_source_negative() {
        let bundle = ai_inference_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid AI storage bundle");
        assert_eq!(bundle.owner_id, "ai");
        assert_eq!(bundle.revision, 6);
        assert_eq!(bundle.steps.len(), 6);
        let sql = std::str::from_utf8(AI_INFERENCE_SCHEMA_V1).expect("utf8");
        let summary_sql = std::str::from_utf8(AI_SUMMARY_SCHEMA_V1).expect("utf8");
        let translation_sql = std::str::from_utf8(AI_TRANSLATION_SCHEMA_V1).expect("utf8");
        let explanation_sql = std::str::from_utf8(AI_EXPLANATION_SCHEMA_V1).expect("utf8");
        let attachment_translation_sql =
            std::str::from_utf8(AI_ATTACHMENT_TRANSLATION_SCHEMA_V1).expect("utf8");
        for required in [
            "ai_inference_runs",
            "request_digest",
            "source_reference_id",
            "source_sha256",
            "selected_provider_settings_revision",
            "result_body_utf8",
            "result_prompt_policy_sha256",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for required in ["ai_summary_runs", "result_summary_utf8", "requested_length"] {
            assert!(summary_sql.contains(required), "{required}");
        }
        for required in [
            "ai_translation_runs",
            "requested_target_language",
            "result_translated_text_utf8",
            "result_detected_source_language",
        ] {
            assert!(translation_sql.contains(required), "{required}");
        }
        for required in [
            "ai_explanation_runs",
            "maximum_reasons",
            "result_exact_bytes",
        ] {
            assert!(explanation_sql.contains(required), "{required}");
        }
        for required in [
            "ai_attachment_translation_runs",
            "result_translated_text_utf8",
            "requested_target_language",
        ] {
            assert!(attachment_translation_sql.contains(required), "{required}");
        }
        for forbidden in [
            "communications_",
            "mail_",
            "telegram_",
            "whatsapp_",
            "zulip_",
            "message_body",
            "provider_id",
            "model_id",
            "endpoint",
            "prompt_text",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
            assert!(!summary_sql.contains(forbidden), "{forbidden}");
            assert!(!translation_sql.contains(forbidden), "{forbidden}");
            assert!(!explanation_sql.contains(forbidden), "{forbidden}");
            assert!(
                !attachment_translation_sql.contains(forbidden),
                "{forbidden}"
            );
        }
        let rls_sql = std::str::from_utf8(AI_INFERENCE_OWNER_RLS_SCHEMA_V1).expect("RLS UTF-8");
        for table in [
            "ai_inference_runs",
            "ai_summary_runs",
            "ai_translation_runs",
            "ai_explanation_runs",
            "ai_attachment_translation_runs",
        ] {
            assert!(rls_sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
            )));
            assert!(rls_sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
            )));
        }
        assert_eq!(rls_sql.matches("CREATE POLICY ").count(), 5);
        assert_eq!(
            rls_sql
                .matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            10
        );
    }
}
