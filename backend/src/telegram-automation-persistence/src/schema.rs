//! Immutable Telegram automation storage migration owned by the integration.

use makosh_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const TELEGRAM_AUTOMATION_STORAGE_REVISION_V1: u32 = 2;

pub const TELEGRAM_AUTOMATION_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_templates (
    template_id TEXT PRIMARY KEY,
    revision BIGINT NOT NULL CHECK (revision > 0),
    name TEXT NOT NULL,
    body_template TEXT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_template_variables (
    template_id TEXT NOT NULL REFERENCES makosh_data.telegram_automation_templates(template_id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    variable_name TEXT NOT NULL,
    PRIMARY KEY (template_id, ordinal),
    UNIQUE (template_id, variable_name)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_policies (
    policy_id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES makosh_data.telegram_automation_templates(template_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL,
    account_id TEXT NOT NULL REFERENCES makosh_data.telegram_accounts(account_id),
    expires_at_unix_seconds BIGINT NULL CHECK (expires_at_unix_seconds IS NULL OR expires_at_unix_seconds > 0),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_policy_chat_scopes (
    policy_id TEXT NOT NULL REFERENCES makosh_data.telegram_automation_policies(policy_id) ON DELETE CASCADE,
    provider_chat_id TEXT NOT NULL,
    PRIMARY KEY (policy_id, provider_chat_id)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_mutation_receipts (
    mutation_id TEXT PRIMARY KEY,
    mutation_kind TEXT NOT NULL,
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    response_payload BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_automation_preview_receipts (
    preview_id TEXT PRIMARY KEY,
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    policy_id TEXT NOT NULL,
    policy_revision BIGINT NOT NULL CHECK (policy_revision > 0),
    template_id TEXT NOT NULL,
    template_revision BIGINT NOT NULL CHECK (template_revision > 0),
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    rendered_text TEXT NOT NULL,
    rendered_sha256 BYTEA NOT NULL CHECK (octet_length(rendered_sha256) = 32),
    response_payload BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0)
);

CREATE INDEX IF NOT EXISTS telegram_automation_policies_account_idx
    ON makosh_data.telegram_automation_policies (account_id, policy_id);
"#;

#[must_use]
pub fn telegram_automation_storage_migration_v1() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_AUTOMATION_STORAGE_REVISION_V1,
        migration_id: "telegram_automation_management_preview".to_owned(),
        forward_sql_utf8: TELEGRAM_AUTOMATION_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_AUTOMATION_SCHEMA_V1.as_bytes()).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_revisioned_and_contains_only_telegram_owned_objects() {
        let migration = telegram_automation_storage_migration_v1();

        assert_eq!(migration.revision, 2);
        assert_eq!(
            migration.migration_id,
            "telegram_automation_management_preview"
        );
        assert!(
            TELEGRAM_AUTOMATION_SCHEMA_V1
                .lines()
                .filter(|line| line.trim_start().starts_with("CREATE TABLE"))
                .all(|line| line.contains("makosh_data.telegram_automation_"))
        );
        assert!(!TELEGRAM_AUTOMATION_SCHEMA_V1.contains("communications_"));
        assert_eq!(
            migration.sha256,
            Sha256::digest(&migration.forward_sql_utf8).to_vec()
        );
    }
}
