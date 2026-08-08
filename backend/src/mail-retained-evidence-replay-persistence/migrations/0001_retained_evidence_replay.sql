CREATE TABLE makosh_data.mail_retained_evidence_replay_index (
    attachment_anchor_id BYTEA PRIMARY KEY CHECK (octet_length(attachment_anchor_id) = 16),
    message_id BYTEA NOT NULL UNIQUE REFERENCES makosh_data.mail_attachment_security_outbox (
        message_id
    ),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    contract_owner TEXT NOT NULL CHECK (contract_owner = 'attachment_security'),
    contract_name TEXT NOT NULL CHECK (
        contract_name = 'attachment_security_scan_candidate_observed'
    ),
    contract_major INTEGER NOT NULL CHECK (contract_major = 1),
    contract_revision INTEGER NOT NULL CHECK (contract_revision = 2),
    contract_schema_sha256 BYTEA NOT NULL CHECK (
        octet_length(contract_schema_sha256) = 32
    ),
    indexed_at_unix_seconds BIGINT NOT NULL CHECK (indexed_at_unix_seconds > 0)
);

CREATE TABLE makosh_data.mail_retained_evidence_replay_audit (
    audit_sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    operation_id BYTEA NOT NULL CHECK (octet_length(operation_id) = 16),
    logical_owner_id TEXT NOT NULL CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    owner_device_actor_sha256 BYTEA NOT NULL CHECK (
        octet_length(owner_device_actor_sha256) = 32
    ),
    producer_registration_id TEXT NOT NULL CHECK (
        length(producer_registration_id) BETWEEN 1 AND 128
    ),
    producer_runtime_generation BIGINT NOT NULL CHECK (producer_runtime_generation > 0),
    producer_grant_epoch BIGINT NOT NULL CHECK (producer_grant_epoch > 0),
    logical_attempt INTEGER NOT NULL CHECK (logical_attempt BETWEEN 1 AND 1024),
    original_message_id BYTEA NOT NULL CHECK (octet_length(original_message_id) = 16),
    original_envelope_sha256 BYTEA NOT NULL CHECK (
        octet_length(original_envelope_sha256) = 32
    ),
    phase SMALLINT NOT NULL CHECK (phase BETWEEN 1 AND 3),
    recorded_at_unix_seconds BIGINT NOT NULL CHECK (recorded_at_unix_seconds > 0),
    UNIQUE (operation_id, original_message_id, logical_attempt, phase)
);

CREATE INDEX mail_retained_evidence_replay_audit_operation_idx
ON makosh_data.mail_retained_evidence_replay_audit (
    operation_id,
    original_message_id,
    audit_sequence
);
