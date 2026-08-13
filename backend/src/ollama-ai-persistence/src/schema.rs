use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1: u32 = 5;
pub const OLLAMA_AI_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_ollama_ai_runs.sql");
pub const OLLAMA_AI_SUMMARY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_ollama_ai_summary_runs.sql");
pub const OLLAMA_AI_TRANSLATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_ollama_ai_translation_runs.sql");
pub const OLLAMA_AI_EXPLANATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0004_ollama_ai_explanation_runs.sql");
pub const OLLAMA_AI_OWNER_RLS_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0005_ollama_ai_owner_rls.sql");

#[must_use]
pub fn ollama_ai_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "ollama_ai_runs".to_owned(),
        owner_id: "ollama".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "ollama_ai_runs_initial".to_owned(),
                forward_sql_utf8: OLLAMA_AI_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OLLAMA_AI_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "ollama_ai_summary_runs".to_owned(),
                forward_sql_utf8: OLLAMA_AI_SUMMARY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OLLAMA_AI_SUMMARY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 3,
                migration_id: "ollama_ai_translation_runs".to_owned(),
                forward_sql_utf8: OLLAMA_AI_TRANSLATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OLLAMA_AI_TRANSLATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 4,
                migration_id: "ollama_ai_explanation_runs".to_owned(),
                forward_sql_utf8: OLLAMA_AI_EXPLANATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OLLAMA_AI_EXPLANATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 5,
                migration_id: "ollama_ai_owner_rls".to_owned(),
                forward_sql_utf8: OLLAMA_AI_OWNER_RLS_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OLLAMA_AI_OWNER_RLS_SCHEMA_V1).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn schema_is_ollama_owned_and_never_persists_private_input() {
        let bundle = ollama_ai_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("storage bundle");
        assert_eq!(bundle.owner_id, "ollama");
        assert_eq!(bundle.revision, 5);
        assert_eq!(bundle.steps.len(), 5);
        let sql = std::str::from_utf8(OLLAMA_AI_SCHEMA_V1).expect("schema");
        let summary_sql = std::str::from_utf8(OLLAMA_AI_SUMMARY_SCHEMA_V1).expect("summary schema");
        let translation_sql =
            std::str::from_utf8(OLLAMA_AI_TRANSLATION_SCHEMA_V1).expect("translation schema");
        let explanation_sql =
            std::str::from_utf8(OLLAMA_AI_EXPLANATION_SCHEMA_V1).expect("explanation schema");
        for required in [
            "request_digest",
            "settings_revision",
            "result_model_revision_sha256",
            "result_body_utf8",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for required in ["ollama_ai_summary_runs", "result_summary_utf8"] {
            assert!(summary_sql.contains(required), "{required}");
        }
        for required in [
            "ollama_ai_translation_runs",
            "result_translated_text_utf8",
            "result_target_language",
        ] {
            assert!(translation_sql.contains(required), "{required}");
        }
        for required in ["ollama_ai_explanation_runs", "result_exact_bytes"] {
            assert!(explanation_sql.contains(required), "{required}");
        }
        for forbidden in [
            "prompt",
            "input_utf8",
            "http_body",
            "endpoint",
            "credential",
            "communications_",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
            assert!(!summary_sql.contains(forbidden), "{forbidden}");
            assert!(!translation_sql.contains(forbidden), "{forbidden}");
            assert!(!explanation_sql.contains(forbidden), "{forbidden}");
        }
        let rls_sql = std::str::from_utf8(OLLAMA_AI_OWNER_RLS_SCHEMA_V1).expect("RLS UTF-8");
        for table in [
            "ollama_ai_runs",
            "ollama_ai_summary_runs",
            "ollama_ai_translation_runs",
            "ollama_ai_explanation_runs",
        ] {
            assert!(rls_sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
            )));
            assert!(rls_sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
            )));
        }
        assert_eq!(rls_sql.matches("CREATE POLICY ").count(), 4);
        assert_eq!(
            rls_sql
                .matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            8
        );
    }
}
