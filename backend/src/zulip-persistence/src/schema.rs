//! Immutable Zulip-owned schema bundle for independent Storage admission.

use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

use crate::ZULIP_SCHEMA_V1;

pub const ZULIP_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V6: u32 = 6;
pub const ZULIP_STORAGE_BUNDLE_REVISION_V7: u32 = 7;

pub const ZULIP_SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_account_state (
    account_id TEXT PRIMARY KEY,
    history_state SMALLINT NOT NULL DEFAULT 1,
    oldest_provider_message_id TEXT,
    last_provider_event_id BIGINT,
    projection_ready BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (length(trim(account_id)) > 0),
    CHECK (history_state BETWEEN 1 AND 4),
    CHECK (oldest_provider_message_id IS NULL OR length(trim(oldest_provider_message_id)) > 0),
    CHECK (last_provider_event_id IS NULL OR last_provider_event_id > 0),
    CHECK (updated_at_unix_seconds > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_events (
    sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider_event_id BIGINT NOT NULL,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT,
    event_kind SMALLINT NOT NULL,
    exact_event_bytes BYTEA NOT NULL,
    event_sha256 BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    UNIQUE (account_id, provider_event_id, provider_message_id, event_kind),
    CHECK (length(trim(account_id)) > 0),
    CHECK (provider_event_id > 0),
    CHECK (length(trim(provider_message_id)) > 0),
    CHECK (event_kind BETWEEN 1 AND 5),
    CHECK (octet_length(exact_event_bytes) BETWEEN 1 AND 524288),
    CHECK (octet_length(event_sha256) = 32),
    CHECK (observed_at_unix_seconds > 0)
);
CREATE INDEX IF NOT EXISTS zulip_operational_events_account_sequence_idx
    ON makosh_data.zulip_operational_events (account_id, sequence DESC);
CREATE INDEX IF NOT EXISTS zulip_operational_events_account_kind_sequence_idx
    ON makosh_data.zulip_operational_events (account_id, event_kind, sequence DESC);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_messages (
    account_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    provider_conversation_id TEXT NOT NULL,
    conversation_kind SMALLINT NOT NULL,
    stream_id TEXT,
    stream_name TEXT,
    topic TEXT,
    direct_recipient_id TEXT,
    sender_id TEXT NOT NULL,
    is_outgoing BOOLEAN NOT NULL,
    content TEXT,
    sent_at_unix_seconds BIGINT,
    edited_at_unix_seconds BIGINT,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    last_event_sequence BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, provider_message_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_message_id)) > 0),
    CHECK (length(trim(provider_conversation_id)) > 0),
    CHECK (conversation_kind IN (1, 2)),
    CHECK (length(trim(sender_id)) > 0),
    CHECK (octet_length(content) <= 1048576),
    CHECK (last_event_sequence >= 0)
);
CREATE INDEX IF NOT EXISTS zulip_operational_messages_account_order_idx
    ON makosh_data.zulip_operational_messages
    (account_id, last_event_sequence DESC, ((provider_message_id)::BIGINT) DESC);
CREATE INDEX IF NOT EXISTS zulip_operational_messages_conversation_order_idx
    ON makosh_data.zulip_operational_messages
    (account_id, provider_conversation_id, last_event_sequence DESC, ((provider_message_id)::BIGINT) DESC);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_message_mutations (
    account_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    content TEXT,
    topic TEXT,
    edited_at_unix_seconds BIGINT,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    last_event_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_message_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_message_id)) > 0),
    CHECK (octet_length(content) <= 1048576),
    CHECK (last_event_sequence > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_conversations (
    account_id TEXT NOT NULL,
    provider_conversation_id TEXT NOT NULL,
    conversation_kind SMALLINT NOT NULL,
    stream_id TEXT,
    stream_name TEXT,
    topic TEXT,
    direct_recipient_id TEXT,
    latest_provider_message_id TEXT,
    latest_event_sequence BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, provider_conversation_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_conversation_id)) > 0),
    CHECK (conversation_kind IN (1, 2)),
    CHECK (latest_event_sequence >= 0)
);
CREATE INDEX IF NOT EXISTS zulip_operational_conversations_account_order_idx
    ON makosh_data.zulip_operational_conversations
    (account_id, latest_event_sequence DESC, ((COALESCE(latest_provider_message_id, '0'))::BIGINT) DESC);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_attachments (
    account_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    provider_attachment_id TEXT NOT NULL,
    filename TEXT,
    PRIMARY KEY (account_id, provider_message_id, provider_attachment_id),
    FOREIGN KEY (account_id, provider_message_id)
      REFERENCES makosh_data.zulip_operational_messages(account_id, provider_message_id)
      ON DELETE CASCADE,
    CHECK (length(trim(provider_attachment_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_operational_reactions (
    account_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    emoji_name TEXT NOT NULL,
    emoji_code TEXT NOT NULL DEFAULT '',
    reaction_type TEXT NOT NULL DEFAULT '',
    present BOOLEAN NOT NULL,
    last_event_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_message_id, actor_id, emoji_name, emoji_code, reaction_type),
    CHECK (length(trim(actor_id)) > 0),
    CHECK (length(trim(emoji_name)) > 0),
    CHECK (last_event_sequence >= 0)
);
"#;

pub const ZULIP_SCHEMA_V3: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.zulip_account_credential_bindings (
    account_id TEXT PRIMARY KEY,
    configuration_instance_id TEXT NOT NULL,
    credential_revision BIGINT NOT NULL,
    binding_revision BIGINT NOT NULL,
    state SMALLINT NOT NULL,
    applied_runtime_generation BIGINT,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(configuration_instance_id)) > 0),
    CHECK (credential_revision > 0),
    CHECK (binding_revision > 0),
    CHECK (state BETWEEN 2 AND 4),
    CHECK (applied_runtime_generation IS NULL OR applied_runtime_generation > 0),
    CHECK ((state = 3 AND applied_runtime_generation IS NOT NULL)
        OR (state IN (2, 4) AND applied_runtime_generation IS NULL)),
    CHECK (updated_at_unix_seconds > 0)
);
"#;

/// Returns the complete Zulip schema as one immutable initial Storage bundle.
///
/// Zulip remains an integration owner: this bundle has no
/// Communications-owned SQL, cross-owner foreign keys, or runtime dependency.
/// Storage Control admits it separately from the Communications inventory.
#[must_use]
pub fn zulip_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ZULIP_STORAGE_BUNDLE_REVISION_V6,
        bundle_id: "zulip_state".to_owned(),
        owner_id: "zulip".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "zulip_state_initial".to_owned(),
                forward_sql_utf8: ZULIP_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(ZULIP_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "zulip_operational_projection".to_owned(),
                forward_sql_utf8: ZULIP_SCHEMA_V2.as_bytes().to_vec(),
                sha256: Sha256::digest(ZULIP_SCHEMA_V2.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "zulip_account_credential_binding".to_owned(),
                forward_sql_utf8: ZULIP_SCHEMA_V3.as_bytes().to_vec(),
                sha256: Sha256::digest(ZULIP_SCHEMA_V3.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "zulip_delivery_route_locators".to_owned(),
                forward_sql_utf8: crate::ZULIP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(crate::ZULIP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "zulip_delivery_intent_inbox_and_jobs".to_owned(),
                forward_sql_utf8: crate::ZULIP_DELIVERY_INTENT_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(crate::ZULIP_DELIVERY_INTENT_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: ZULIP_STORAGE_BUNDLE_REVISION_V6,
                migration_id: "zulip_delivery_intent_result_outbox".to_owned(),
                forward_sql_utf8: crate::ZULIP_DELIVERY_INTENT_RESULT_OUTBOX_SCHEMA_V1
                    .as_bytes()
                    .to_vec(),
                sha256: Sha256::digest(
                    crate::ZULIP_DELIVERY_INTENT_RESULT_OUTBOX_SCHEMA_V1.as_bytes(),
                )
                .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owned_only_by_zulip() {
        let bundle = zulip_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "zulip");
        assert_eq!(bundle.bundle_id, "zulip_state");
        assert_eq!(bundle.revision, ZULIP_STORAGE_BUNDLE_REVISION_V6);
        assert_eq!(bundle.steps.len(), 6);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(bundle.steps[0].forward_sql_utf8, ZULIP_SCHEMA_V1.as_bytes());
        assert_eq!(bundle.steps[1].forward_sql_utf8, ZULIP_SCHEMA_V2.as_bytes());
        assert_eq!(bundle.steps[2].forward_sql_utf8, ZULIP_SCHEMA_V3.as_bytes());
        assert_eq!(
            bundle.steps[3].forward_sql_utf8,
            crate::ZULIP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes()
        );
        assert_eq!(
            bundle.steps[4].forward_sql_utf8,
            crate::ZULIP_DELIVERY_INTENT_SCHEMA_V1.as_bytes()
        );
        assert_eq!(
            bundle.steps[5].forward_sql_utf8,
            crate::ZULIP_DELIVERY_INTENT_RESULT_OUTBOX_SCHEMA_V1.as_bytes()
        );
        let sql = std::str::from_utf8(&bundle.steps[1].forward_sql_utf8)
            .expect("Zulip Storage SQL is UTF-8");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 7);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS makosh_data.")
                .count(),
            7,
            "every Zulip table belongs to the owner-scoped makosh_data schema"
        );
        assert!(!sql.contains("makosh_communications"));
        assert!(!sql.contains("REFERENCES communications_"));
        let account_sql = std::str::from_utf8(&bundle.steps[2].forward_sql_utf8)
            .expect("Zulip account binding SQL is UTF-8");
        assert!(!account_sql.contains("secret_ref"));
        assert!(!account_sql.contains("api_key"));
    }
}
