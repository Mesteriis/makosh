ALTER TABLE makosh_data.attachment_translation_runs
ADD COLUMN artifact_runtime_generation BIGINT,
ADD COLUMN artifact_grant_epoch BIGINT,
ADD CONSTRAINT attachment_translation_artifact_fence_check CHECK (
    (artifact_id IS NULL AND artifact_runtime_generation IS NULL
      AND artifact_grant_epoch IS NULL)
    OR (artifact_id IS NOT NULL AND artifact_runtime_generation > 0
      AND artifact_grant_epoch > 0)
);

CREATE TABLE makosh_data.attachment_translation_read_tickets (
    logical_owner_id TEXT NOT NULL,
    ticket_sha256 BYTEA NOT NULL,
    device_actor_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    state_revision BIGINT NOT NULL,
    artifact_reference_id BYTEA NOT NULL,
    artifact_receipt_sha256 BYTEA NOT NULL,
    translated_size_bytes BIGINT NOT NULL,
    runtime_generation BIGINT NOT NULL,
    grant_epoch BIGINT NOT NULL,
    expires_at_unix_seconds BIGINT NOT NULL,
    used_at_unix_seconds BIGINT,
    created_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, ticket_sha256),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(ticket_sha256) = 32),
    CHECK (length(device_actor_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (state_revision > 0),
    CHECK (length(artifact_reference_id) = 16),
    CHECK (length(artifact_receipt_sha256) = 32),
    CHECK (translated_size_bytes BETWEEN 1 AND 65536),
    CHECK (runtime_generation > 0),
    CHECK (grant_epoch > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (expires_at_unix_seconds >= created_at_unix_seconds),
    CHECK (used_at_unix_seconds IS NULL
      OR used_at_unix_seconds >= created_at_unix_seconds)
);

CREATE INDEX attachment_translation_read_tickets_expiry_idx
ON makosh_data.attachment_translation_read_tickets (
    logical_owner_id,
    expires_at_unix_seconds
);
