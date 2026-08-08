//! Immutable WhatsApp-owned schema bundle for independent Storage admission.

use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const WHATSAPP_STORAGE_BUNDLE_REVISION_V4: u32 = 4;

pub const WHATSAPP_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS whatsapp_communications_outbox_pending_idx
    ON makosh_data.whatsapp_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_host_observations (
    account_id TEXT NOT NULL,
    provider_event_id TEXT NOT NULL,
    evidence_kind SMALLINT NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_event_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_event_id) BETWEEN 1 AND 256),
    CHECK (evidence_kind BETWEEN 1 AND 11)
);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_provider_commands (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    exact_command_bytes BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    host_claim_id TEXT,
    lease_expires_at_unix_seconds BIGINT,
    requested_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    CHECK (char_length(operation_id) BETWEEN 1 AND 256),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 524288),
    CHECK (state BETWEEN 1 AND 4),
    CHECK ((state = 1 AND host_claim_id IS NULL AND lease_expires_at_unix_seconds IS NULL AND completed_at_unix_seconds IS NULL)
        OR (state = 2 AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NULL)
        OR (state IN (3, 4) AND host_claim_id IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL AND completed_at_unix_seconds IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS whatsapp_provider_commands_claimable_idx
    ON makosh_data.whatsapp_provider_commands (account_id, requested_at_unix_seconds, operation_id)
    WHERE state IN (1, 2);
"#;

pub const WHATSAPP_SCHEMA_V2: &str = r#"
ALTER TABLE makosh_data.whatsapp_host_observations
    ADD COLUMN IF NOT EXISTS operational_sha256 BYTEA;
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_events (
    sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider_event_id TEXT NOT NULL,
    event_kind SMALLINT NOT NULL,
    provider_chat_id TEXT,
    exact_event_bytes BYTEA NOT NULL,
    event_sha256 BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    UNIQUE (account_id, provider_event_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_event_id) BETWEEN 1 AND 256),
    CHECK (event_kind BETWEEN 1 AND 17),
    CHECK (provider_chat_id IS NULL OR char_length(provider_chat_id) BETWEEN 1 AND 256),
    CHECK (octet_length(exact_event_bytes) BETWEEN 1 AND 524288),
    CHECK (octet_length(event_sha256) = 32)
);
CREATE INDEX IF NOT EXISTS whatsapp_operational_events_account_sequence_idx
    ON makosh_data.whatsapp_operational_events (account_id, sequence DESC);
CREATE INDEX IF NOT EXISTS whatsapp_operational_events_account_kind_sequence_idx
    ON makosh_data.whatsapp_operational_events (account_id, event_kind, sequence DESC);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_messages (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    sender_display_name TEXT NOT NULL,
    body_text TEXT,
    reply_to_provider_message_id TEXT,
    delivery_state TEXT,
    occurred_at_unix_seconds BIGINT NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    last_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, provider_message_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_message_id) BETWEEN 1 AND 256),
    CHECK (char_length(sender_id) BETWEEN 1 AND 256),
    CHECK (octet_length(body_text) <= 262144)
);
CREATE INDEX IF NOT EXISTS whatsapp_operational_messages_account_sequence_idx
    ON makosh_data.whatsapp_operational_messages (account_id, last_sequence DESC);
CREATE INDEX IF NOT EXISTS whatsapp_operational_messages_chat_sequence_idx
    ON makosh_data.whatsapp_operational_messages (account_id, provider_chat_id, last_sequence DESC);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_dialogs (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    title TEXT NOT NULL,
    dialog_kind TEXT NOT NULL,
    is_archived BOOLEAN,
    is_pinned BOOLEAN,
    is_muted BOOLEAN,
    is_unread BOOLEAN,
    unread_count BIGINT,
    participant_count BIGINT,
    observed_at_unix_seconds BIGINT NOT NULL,
    last_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 256),
    CHECK (unread_count >= 0),
    CHECK (participant_count >= 0)
);
CREATE INDEX IF NOT EXISTS whatsapp_operational_dialogs_account_sequence_idx
    ON makosh_data.whatsapp_operational_dialogs (account_id, last_sequence DESC);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_participants (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_identity_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    participant_role TEXT NOT NULL,
    participant_status TEXT NOT NULL,
    is_self BOOLEAN NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    last_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, provider_identity_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_identity_id) BETWEEN 1 AND 256)
);
CREATE INDEX IF NOT EXISTS whatsapp_operational_participants_chat_sequence_idx
    ON makosh_data.whatsapp_operational_participants (account_id, provider_chat_id, last_sequence DESC);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_tombstones (
    account_id TEXT NOT NULL,
    entity_kind SMALLINT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_entity_id TEXT NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    last_sequence BIGINT NOT NULL,
    PRIMARY KEY (account_id, entity_kind, provider_chat_id, provider_entity_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (entity_kind IN (1, 2)),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_entity_id) BETWEEN 1 AND 256),
    CHECK (observed_at_unix_seconds > 0),
    CHECK (last_sequence > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_runtime_status (
    account_id TEXT PRIMARY KEY,
    runtime_state TEXT,
    projection_ready BOOLEAN NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    last_sequence BIGINT NOT NULL,
    CHECK (char_length(account_id) BETWEEN 1 AND 256)
);
CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_operational_controls (
    account_id TEXT NOT NULL,
    provider_event_id TEXT NOT NULL,
    control_kind SMALLINT NOT NULL,
    content_sha256 BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (account_id, provider_event_id),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_event_id) BETWEEN 1 AND 256),
    CHECK (control_kind = 1),
    CHECK (octet_length(content_sha256) = 32)
);
"#;

/// Returns the complete WhatsApp schema as one immutable ordered bundle.
///
/// The bundle remains owned by the integration and contains no Communications
/// tables, cross-owner foreign keys, credentials, or provider session state.
#[must_use]
pub fn whatsapp_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V4,
        bundle_id: "whatsapp_state".to_owned(),
        owner_id: "whatsapp".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "whatsapp_state_initial".to_owned(),
                forward_sql_utf8: WHATSAPP_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(WHATSAPP_SCHEMA_V1.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "whatsapp_operational_read".to_owned(),
                forward_sql_utf8: WHATSAPP_SCHEMA_V2.as_bytes().to_vec(),
                sha256: Sha256::digest(WHATSAPP_SCHEMA_V2.as_bytes()).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "whatsapp_delivery_route_locator".to_owned(),
                forward_sql_utf8: crate::WHATSAPP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes().to_vec(),
                sha256: Sha256::digest(crate::WHATSAPP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes())
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: WHATSAPP_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "whatsapp_delivery_intent_inbox_jobs_and_result_outbox".to_owned(),
                forward_sql_utf8: crate::WHATSAPP_DELIVERY_INTENT_SCHEMA_V1
                    .as_bytes()
                    .to_vec(),
                sha256: Sha256::digest(crate::WHATSAPP_DELIVERY_INTENT_SCHEMA_V1.as_bytes())
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
    fn bundle_is_valid_and_owned_only_by_whatsapp() {
        let bundle = whatsapp_storage_bundle_v1();

        assert_eq!(bundle.owner_id, "whatsapp");
        assert_eq!(bundle.bundle_id, "whatsapp_state");
        assert_eq!(bundle.revision, WHATSAPP_STORAGE_BUNDLE_REVISION_V4);
        assert_eq!(bundle.steps.len(), 4);
        assert_eq!(validate_storage_bundle(&bundle), Ok(()));
        assert_eq!(
            bundle.steps[0].forward_sql_utf8,
            WHATSAPP_SCHEMA_V1.as_bytes()
        );
        assert_eq!(
            bundle.steps[1].forward_sql_utf8,
            WHATSAPP_SCHEMA_V2.as_bytes()
        );
        assert_eq!(
            bundle.steps[2].forward_sql_utf8,
            crate::WHATSAPP_DELIVERY_ROUTE_SCHEMA_V1.as_bytes()
        );
        assert_eq!(
            bundle.steps[3].forward_sql_utf8,
            crate::WHATSAPP_DELIVERY_INTENT_SCHEMA_V1.as_bytes()
        );
        let sql = bundle
            .steps
            .iter()
            .map(|step| std::str::from_utf8(&step.forward_sql_utf8).expect("WhatsApp SQL is UTF-8"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(sql.matches("CREATE TABLE IF NOT EXISTS ").count(), 16);
        assert_eq!(
            sql.matches("CREATE TABLE IF NOT EXISTS makosh_data.")
                .count(),
            16
        );
        assert!(!sql.contains("makosh_data.communications_"));
        assert!(!sql.contains("REFERENCES makosh_data.communications_"));
        for forbidden in ["INSERT ", "UPDATE ", "DELETE "] {
            assert!(!WHATSAPP_SCHEMA_V2.contains(forbidden));
        }
    }
}
