use makosh_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const TELEGRAM_CALLS_STORAGE_REVISION_V1: u32 = 3;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V2: u32 = 4;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V3: u32 = 5;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V4: u32 = 6;
pub const TELEGRAM_CALLS_STORAGE_REVISION_V5: u32 = 9;

pub const TELEGRAM_CALLS_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_sessions (
    call_session_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES makosh_data.telegram_accounts(account_id),
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    tdlib_call_id INTEGER NOT NULL CHECK (tdlib_call_id > 0),
    provider_call_unique_id BIGINT NULL CHECK (provider_call_unique_id IS NULL OR provider_call_unique_id > 0),
    provider_user_id TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('incoming', 'outgoing')),
    provider_state TEXT NOT NULL CHECK (provider_state IN ('pending', 'exchanging_keys', 'media_ready', 'hanging_up', 'discarded', 'error')),
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL CHECK (discard_reason IS NULL OR discard_reason IN ('empty', 'missed', 'declined', 'disconnected', 'hung_up')),
    failure_category TEXT NULL CHECK (failure_category IS NULL OR failure_category IN ('network', 'not_available', 'permission', 'protocol', 'unknown')),
    revision BIGINT NOT NULL CHECK (revision > 0),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    ended_at_unix_seconds BIGINT NULL CHECK (ended_at_unix_seconds IS NULL OR ended_at_unix_seconds > 0),
    UNIQUE (account_id, runtime_generation, tdlib_call_id),
    UNIQUE (account_id, provider_call_unique_id)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_state_history (
    call_session_id TEXT NOT NULL REFERENCES makosh_data.telegram_call_sessions(call_session_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    provider_state TEXT NOT NULL,
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL,
    failure_category TEXT NULL,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    PRIMARY KEY (call_session_id, revision)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_realtime_frames (
    frame_sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    call_session_id TEXT NOT NULL REFERENCES makosh_data.telegram_call_sessions(call_session_id),
    call_revision BIGINT NOT NULL CHECK (call_revision > 0),
    provider_state TEXT NOT NULL,
    pending_created BOOLEAN NOT NULL,
    pending_received BOOLEAN NOT NULL,
    discard_reason TEXT NULL,
    failure_category TEXT NULL,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    UNIQUE (call_session_id, call_revision)
);

CREATE INDEX IF NOT EXISTS telegram_call_sessions_account_idx
    ON makosh_data.telegram_call_sessions (account_id, call_session_id);

CREATE INDEX IF NOT EXISTS telegram_call_realtime_account_sequence_idx
    ON makosh_data.telegram_call_realtime_frames (account_id, frame_sequence);
"#;

pub const TELEGRAM_CALLS_SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_operations (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES makosh_data.telegram_accounts(account_id),
    call_session_id TEXT NOT NULL,
    operation_kind TEXT NOT NULL CHECK (operation_kind IN (
        'initiate_audio', 'accept_audio', 'decline', 'end', 'set_local_mute'
    )),
    operation_state TEXT NOT NULL CHECK (operation_state IN (
        'accepted', 'dispatching', 'awaiting_provider', 'completed', 'failed'
    )),
    request_fingerprint_sha256 BYTEA NOT NULL
        CHECK (octet_length(request_fingerprint_sha256) = 32),
    provider_user_id TEXT NULL,
    requested_mute BOOLEAN NULL,
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    grant_epoch BIGINT NOT NULL CHECK (grant_epoch > 0),
    tdlib_call_id INTEGER NULL CHECK (tdlib_call_id IS NULL OR tdlib_call_id > 0),
    revision BIGINT NOT NULL CHECK (revision > 0),
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT NULL
        CHECK (completed_at_unix_seconds IS NULL OR completed_at_unix_seconds > 0),
    failure_category TEXT NULL CHECK (
        failure_category IS NULL OR failure_category IN (
            'network', 'not_available', 'permission', 'protocol', 'unknown'
        )
    ),
    CHECK ((operation_kind = 'initiate_audio') = (provider_user_id IS NOT NULL)),
    CHECK ((operation_kind = 'set_local_mute') = (requested_mute IS NOT NULL)),
    CHECK ((operation_state = 'failed') = (failure_category IS NOT NULL)),
    CHECK (
        (operation_state IN ('completed', 'failed')) =
        (completed_at_unix_seconds IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_local_mute (
    call_session_id TEXT PRIMARY KEY
        REFERENCES makosh_data.telegram_call_sessions(call_session_id),
    account_id TEXT NOT NULL,
    muted BOOLEAN NOT NULL,
    operation_id TEXT NOT NULL
        REFERENCES makosh_data.telegram_call_operations(operation_id),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_operation_history (
    operation_id TEXT NOT NULL
        REFERENCES makosh_data.telegram_call_operations(operation_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    operation_state TEXT NOT NULL,
    tdlib_call_id INTEGER NULL,
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT NULL,
    failure_category TEXT NULL,
    PRIMARY KEY (operation_id, revision)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_realtime_events (
    event_sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id TEXT NOT NULL,
    event_kind TEXT NOT NULL CHECK (event_kind IN ('call', 'operation')),
    call_session_id TEXT NULL,
    call_revision BIGINT NULL CHECK (call_revision IS NULL OR call_revision > 0),
    operation_id TEXT NULL,
    operation_revision BIGINT NULL
        CHECK (operation_revision IS NULL OR operation_revision > 0),
    local_muted BOOLEAN NOT NULL DEFAULT FALSE,
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    CHECK (
        (event_kind = 'call' AND call_session_id IS NOT NULL
            AND call_revision IS NOT NULL AND operation_id IS NULL
            AND operation_revision IS NULL)
        OR
        (event_kind = 'operation' AND operation_id IS NOT NULL
            AND operation_revision IS NOT NULL AND call_session_id IS NULL
            AND call_revision IS NULL)
    ),
    UNIQUE (call_session_id, call_revision),
    UNIQUE (operation_id, operation_revision)
);

CREATE UNIQUE INDEX IF NOT EXISTS telegram_call_sessions_one_active_per_account_idx
    ON makosh_data.telegram_call_sessions (account_id)
    WHERE provider_state NOT IN ('discarded', 'error');

CREATE UNIQUE INDEX IF NOT EXISTS telegram_call_operations_one_initiate_per_account_idx
    ON makosh_data.telegram_call_operations (account_id)
    WHERE operation_kind = 'initiate_audio'
      AND operation_state NOT IN ('completed', 'failed');

CREATE INDEX IF NOT EXISTS telegram_call_operations_account_id_idx
    ON makosh_data.telegram_call_operations (account_id, operation_id);

CREATE INDEX IF NOT EXISTS telegram_call_realtime_events_account_sequence_idx
    ON makosh_data.telegram_call_realtime_events (account_id, event_sequence);
"#;

pub const TELEGRAM_CALLS_SCHEMA_V3: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_media_projection (
    call_session_id TEXT PRIMARY KEY
        REFERENCES makosh_data.telegram_call_sessions(call_session_id),
    account_id TEXT NOT NULL,
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    provider_revision BIGINT NOT NULL CHECK (provider_revision > 0),
    media_state TEXT NOT NULL CHECK (
        media_state IN ('connecting', 'active', 'reconnecting', 'failed')
    ),
    revision BIGINT NOT NULL CHECK (revision > 0),
    connected_at_unix_seconds BIGINT NULL CHECK (
        connected_at_unix_seconds IS NULL OR connected_at_unix_seconds > 0
    ),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    failed_at_unix_seconds BIGINT NULL CHECK (
        failed_at_unix_seconds IS NULL OR failed_at_unix_seconds > 0
    )
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_media_state_history (
    call_session_id TEXT NOT NULL
        REFERENCES makosh_data.telegram_call_sessions(call_session_id),
    revision BIGINT NOT NULL CHECK (revision > 0),
    runtime_generation BIGINT NOT NULL CHECK (runtime_generation > 0),
    provider_revision BIGINT NOT NULL CHECK (provider_revision > 0),
    media_state TEXT NOT NULL CHECK (
        media_state IN ('connecting', 'active', 'reconnecting', 'failed')
    ),
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    PRIMARY KEY (call_session_id, revision)
);

CREATE INDEX IF NOT EXISTS telegram_call_media_projection_account_idx
    ON makosh_data.telegram_call_media_projection (account_id, call_session_id);
"#;

pub const TELEGRAM_CALLS_SCHEMA_V4: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_realtime_replay_order (
    replay_sequence BIGINT PRIMARY KEY CHECK (replay_sequence > 0),
    event_sequence BIGINT NOT NULL UNIQUE
        REFERENCES makosh_data.telegram_call_realtime_events(event_sequence)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_realtime_replay_cursor (
    cursor_scope TEXT PRIMARY KEY CHECK (cursor_scope = 'owner'),
    next_sequence BIGINT NOT NULL CHECK (next_sequence > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_realtime_backfill_jobs (
    job_run_id BYTEA PRIMARY KEY CHECK (octet_length(job_run_id) = 16),
    job_owner TEXT NOT NULL CHECK (job_owner = 'telegram'),
    job_name TEXT NOT NULL CHECK (job_name = 'calls_realtime_backfill'),
    job_major INTEGER NOT NULL CHECK (job_major = 1),
    scope_id TEXT NOT NULL CHECK (scope_id = 'owner'),
    command_message_id BYTEA NOT NULL UNIQUE
        CHECK (octet_length(command_message_id) = 16),
    command_envelope_bytes BYTEA NOT NULL
        CHECK (
            octet_length(command_envelope_bytes) > 0
            AND octet_length(command_envelope_bytes) <= 262144
        ),
    command_envelope_sha256 BYTEA NOT NULL
        CHECK (octet_length(command_envelope_sha256) = 32),
    execution_state TEXT NOT NULL
        CHECK (execution_state IN ('accepted', 'running', 'succeeded')),
    execution_phase TEXT NOT NULL
        CHECK (execution_phase IN ('pending', 'rebase', 'backfill', 'complete')),
    execution_runtime_generation BIGINT NULL
        CHECK (
            execution_runtime_generation IS NULL
            OR execution_runtime_generation > 0
        ),
    lease_epoch BIGINT NOT NULL CHECK (lease_epoch >= 0),
    lease_expires_at_unix_millis BIGINT NULL
        CHECK (
            lease_expires_at_unix_millis IS NULL
            OR lease_expires_at_unix_millis > 0
        ),
    checkpoint_frame_sequence BIGINT NOT NULL DEFAULT 0
        CHECK (checkpoint_frame_sequence >= 0),
    processed_frame_count BIGINT NOT NULL DEFAULT 0
        CHECK (processed_frame_count >= 0),
    backfilled_frame_count BIGINT NOT NULL DEFAULT 0
        CHECK (
            backfilled_frame_count >= 0
            AND backfilled_frame_count <= processed_frame_count
        ),
    rebase_original_max_event_sequence BIGINT NULL
        CHECK (
            rebase_original_max_event_sequence IS NULL
            OR rebase_original_max_event_sequence >= 0
        ),
    rebase_offset BIGINT NULL
        CHECK (rebase_offset IS NULL OR rebase_offset > 0),
    rebase_mapped_event_count BIGINT NOT NULL DEFAULT 0
        CHECK (rebase_mapped_event_count >= 0),
    attempt_count BIGINT NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    accepted_at_unix_millis BIGINT NOT NULL CHECK (accepted_at_unix_millis > 0),
    updated_at_unix_millis BIGINT NOT NULL CHECK (updated_at_unix_millis > 0),
    completed_at_unix_millis BIGINT NULL
        CHECK (
            completed_at_unix_millis IS NULL
            OR completed_at_unix_millis > 0
        ),
    CHECK (
        (
            execution_state = 'accepted'
            AND execution_phase = 'pending'
            AND execution_runtime_generation IS NULL
            AND lease_epoch = 0
            AND lease_expires_at_unix_millis IS NULL
            AND attempt_count = 0
            AND rebase_original_max_event_sequence IS NULL
            AND rebase_offset IS NULL
            AND rebase_mapped_event_count = 0
            AND completed_at_unix_millis IS NULL
        )
        OR (
            execution_state = 'running'
            AND execution_phase IN ('rebase', 'backfill')
            AND execution_runtime_generation IS NOT NULL
            AND lease_epoch > 0
            AND lease_expires_at_unix_millis IS NOT NULL
            AND attempt_count > 0
            AND rebase_original_max_event_sequence IS NOT NULL
            AND rebase_offset IS NOT NULL
            AND completed_at_unix_millis IS NULL
        )
        OR (
            execution_state = 'succeeded'
            AND execution_phase = 'complete'
            AND execution_runtime_generation IS NOT NULL
            AND lease_epoch > 0
            AND lease_expires_at_unix_millis IS NOT NULL
            AND attempt_count > 0
            AND rebase_original_max_event_sequence IS NOT NULL
            AND rebase_offset IS NOT NULL
            AND completed_at_unix_millis IS NOT NULL
        )
    )
);

CREATE INDEX IF NOT EXISTS telegram_call_realtime_backfill_state_idx
    ON makosh_data.telegram_call_realtime_backfill_jobs (
        execution_state,
        updated_at_unix_millis
    );
"#;

pub const TELEGRAM_CALLS_SCHEMA_V5: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_call_evidence_outbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    exact_envelope_bytes BYTEA NOT NULL CHECK (
        octet_length(exact_envelope_bytes) > 0
        AND octet_length(exact_envelope_bytes) <= 262144
    ),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    published_at_unix_seconds BIGINT NULL CHECK (
        published_at_unix_seconds IS NULL
        OR published_at_unix_seconds > 0
    )
);

CREATE INDEX IF NOT EXISTS telegram_call_evidence_outbox_pending_idx
    ON makosh_data.telegram_call_evidence_outbox (
        created_at_unix_seconds,
        message_id
    )
    WHERE published_at_unix_seconds IS NULL;
"#;

pub fn telegram_calls_storage_migration_v1() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V1,
        migration_id: "telegram_call_history".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V1.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V1.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v2() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V2,
        migration_id: "telegram_call_signaling".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V2.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V2.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v3() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V3,
        migration_id: "telegram_call_media_projection".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V3.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V3.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v4() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V4,
        migration_id: "telegram_call_realtime_backfill_job".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V4.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V4.as_bytes()).to_vec(),
    }
}

pub fn telegram_calls_storage_migration_v5() -> StorageMigrationStepV1 {
    StorageMigrationStepV1 {
        revision: TELEGRAM_CALLS_STORAGE_REVISION_V5,
        migration_id: "telegram_call_evidence_outbox".to_owned(),
        forward_sql_utf8: TELEGRAM_CALLS_SCHEMA_V5.as_bytes().to_vec(),
        sha256: Sha256::digest(TELEGRAM_CALLS_SCHEMA_V5.as_bytes()).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_revisioned_and_owner_local() {
        let migration = telegram_calls_storage_migration_v1();

        assert_eq!(migration.revision, 3);
        assert_eq!(migration.migration_id, "telegram_call_history");
        assert!(TELEGRAM_CALLS_SCHEMA_V1.contains("makosh_data.telegram_call_sessions"));
        assert!(TELEGRAM_CALLS_SCHEMA_V1.contains("telegram_call_realtime_frames"));

        let signaling = telegram_calls_storage_migration_v2();
        assert_eq!(signaling.revision, 4);
        assert_eq!(signaling.migration_id, "telegram_call_signaling");
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_operations"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_local_mute"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_operation_history"));
        assert!(TELEGRAM_CALLS_SCHEMA_V2.contains("telegram_call_realtime_events"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V2.contains("INSERT INTO"));

        let media = telegram_calls_storage_migration_v3();
        assert_eq!(media.revision, 5);
        assert_eq!(media.migration_id, "telegram_call_media_projection");
        assert!(TELEGRAM_CALLS_SCHEMA_V3.contains("telegram_call_media_projection"));
        assert!(TELEGRAM_CALLS_SCHEMA_V3.contains("telegram_call_media_state_history"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V3.contains("INSERT INTO"));

        let backfill = telegram_calls_storage_migration_v4();
        assert_eq!(backfill.revision, 6);
        let evidence = telegram_calls_storage_migration_v5();
        assert_eq!(evidence.revision, 9);
        assert!(TELEGRAM_CALLS_SCHEMA_V5.contains("telegram_call_evidence_outbox"));
        assert_eq!(backfill.migration_id, "telegram_call_realtime_backfill_job");
        assert!(TELEGRAM_CALLS_SCHEMA_V4.contains("telegram_call_realtime_replay_order"));
        assert!(TELEGRAM_CALLS_SCHEMA_V4.contains("telegram_call_realtime_replay_cursor"));
        assert!(TELEGRAM_CALLS_SCHEMA_V4.contains("telegram_call_realtime_backfill_jobs"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V4.contains("INSERT INTO"));
        assert!(!TELEGRAM_CALLS_SCHEMA_V4.contains("UPDATE "));
        assert!(!TELEGRAM_CALLS_SCHEMA_V4.contains("DELETE "));
    }
}
