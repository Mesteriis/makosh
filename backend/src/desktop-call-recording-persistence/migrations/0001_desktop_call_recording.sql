CREATE TABLE makosh_data.desktop_call_recording_runs (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    request_sha256 BYTEA NOT NULL,
    call_evidence_id BYTEA NOT NULL,
    call_evidence_revision BIGINT NOT NULL,
    recording_evidence_id BYTEA NOT NULL,
    recording_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    device_actor_sha256 BYTEA NOT NULL,
    challenge_id BYTEA NOT NULL,
    challenge_expires_at_unix_ms BIGINT NOT NULL,
    maximum_duration_millis BIGINT NOT NULL,
    consent_policy_revision INTEGER NOT NULL,
    started_at_unix_ms BIGINT,
    ended_at_unix_ms BIGINT,
    consent_receipt_id BYTEA,
    source_reference_id BYTEA,
    source_declared_bytes BIGINT,
    source_duration_millis BIGINT,
    source_sha256 BYTEA,
    public_error_code TEXT,
    PRIMARY KEY (logical_owner_id, operation_id),
    UNIQUE (logical_owner_id, recording_evidence_id),
    CHECK (octet_length(operation_id) = 16),
    CHECK (octet_length(request_sha256) = 32),
    CHECK (octet_length(call_evidence_id) = 16),
    CHECK (octet_length(recording_evidence_id) = 16),
    CHECK (octet_length(device_actor_sha256) = 32),
    CHECK (octet_length(challenge_id) = 16),
    CHECK (recording_revision > 0),
    CHECK (call_evidence_revision > 0),
    CHECK (maximum_duration_millis BETWEEN 1000 AND 14400000),
    CHECK (consent_policy_revision > 0),
    CHECK (run_state BETWEEN 1 AND 5),
    CHECK (consent_receipt_id IS NULL OR octet_length(consent_receipt_id) = 16),
    CHECK (source_reference_id IS NULL OR octet_length(source_reference_id) = 16),
    CHECK (source_sha256 IS NULL OR octet_length(source_sha256) = 32),
    CHECK (public_error_code IS NULL OR (length(public_error_code) BETWEEN 1 AND 96))
);

CREATE TABLE makosh_data.desktop_call_recording_host_commands (
    command_id BYTEA PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    recording_evidence_id BYTEA NOT NULL,
    command_kind SMALLINT NOT NULL,
    command_revision BIGINT NOT NULL,
    leased_by_sha256 BYTEA,
    lease_expires_at_unix_ms BIGINT,
    completed_at_unix_ms BIGINT,
    CHECK (octet_length(command_id) = 16),
    CHECK (octet_length(recording_evidence_id) = 16),
    CHECK (command_kind BETWEEN 1 AND 2),
    CHECK (command_revision > 0),
    CHECK (leased_by_sha256 IS NULL OR octet_length(leased_by_sha256) = 32)
);

CREATE INDEX desktop_call_recording_host_commands_claim
ON makosh_data.desktop_call_recording_host_commands (completed_at_unix_ms, lease_expires_at_unix_ms, command_revision);

CREATE TABLE makosh_data.desktop_call_recording_outbox (
    sequence_id BIGSERIAL PRIMARY KEY,
    event_id BYTEA NOT NULL UNIQUE,
    logical_owner_id TEXT NOT NULL,
    recording_evidence_id BYTEA NOT NULL,
    contract_name TEXT NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    delivered_at_unix_ms BIGINT,
    CHECK (octet_length(event_id) = 16),
    CHECK (octet_length(recording_evidence_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) BETWEEN 1 AND 131072)
);

CREATE INDEX desktop_call_recording_outbox_pending
ON makosh_data.desktop_call_recording_outbox (sequence_id) WHERE delivered_at_unix_ms IS NULL;

CREATE TABLE makosh_data.desktop_call_recording_realtime (
    sequence_id BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    recording_evidence_id BYTEA NOT NULL,
    recording_revision BIGINT NOT NULL,
    occurred_at_unix_ms BIGINT NOT NULL,
    payload_bytes BYTEA NOT NULL,
    payload_sha256 BYTEA NOT NULL,
    CHECK (octet_length(recording_evidence_id) = 16),
    CHECK (recording_revision > 0),
    CHECK (octet_length(payload_bytes) BETWEEN 1 AND 4096),
    CHECK (octet_length(payload_sha256) = 32),
    UNIQUE (logical_owner_id, recording_evidence_id, recording_revision)
);
