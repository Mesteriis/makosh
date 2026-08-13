CREATE TABLE makosh_data.relationships_records (
    logical_owner_id TEXT NOT NULL,
    relationship_id BYTEA NOT NULL,
    source_kind SMALLINT NOT NULL,
    source_public_id BYTEA NOT NULL,
    target_kind SMALLINT NOT NULL,
    target_public_id BYTEA NOT NULL,
    relationship_type SMALLINT NOT NULL,
    relationship_state SMALLINT NOT NULL,
    valid_from_unix_seconds BIGINT NOT NULL,
    valid_from_nanos INTEGER NOT NULL,
    valid_until_unix_seconds BIGINT,
    valid_until_nanos INTEGER,
    relationship_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, relationship_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(relationship_id) = 16),
    CHECK (source_kind BETWEEN 1 AND 2),
    CHECK (target_kind BETWEEN 1 AND 2),
    CHECK (length(source_public_id) = 16),
    CHECK (length(target_public_id) = 16),
    CHECK (source_public_id <> target_public_id OR source_kind <> target_kind),
    CHECK (relationship_type BETWEEN 1 AND 6),
    CHECK (relationship_state BETWEEN 1 AND 2),
    CHECK (valid_from_unix_seconds > 0 AND valid_from_nanos BETWEEN 0 AND 999999999),
    CHECK ((valid_until_unix_seconds IS NULL AND valid_until_nanos IS NULL) OR
           (valid_until_unix_seconds > 0 AND valid_until_nanos BETWEEN 0 AND 999999999 AND
            (valid_until_unix_seconds, valid_until_nanos) > (valid_from_unix_seconds, valid_from_nanos))),
    CHECK (relationship_revision > 0),
    CHECK (created_at_unix_seconds > 0 AND created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds > 0 AND updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.relationships_evidence (
    logical_owner_id TEXT NOT NULL,
    relationship_id BYTEA NOT NULL,
    evidence_id BYTEA NOT NULL,
    source_owner_id TEXT NOT NULL,
    source_record_id TEXT NOT NULL,
    source_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    observed_at_nanos INTEGER NOT NULL,
    evidence_state SMALLINT NOT NULL,
    updated_at_relationship_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, relationship_id, evidence_id),
    UNIQUE (logical_owner_id, relationship_id, source_owner_id, source_record_id),
    FOREIGN KEY (logical_owner_id, relationship_id)
        REFERENCES makosh_data.relationships_records (logical_owner_id, relationship_id) ON DELETE CASCADE,
    CHECK (length(evidence_id) = 16),
    CHECK (length(source_owner_id) BETWEEN 1 AND 128),
    CHECK (length(source_record_id) BETWEEN 1 AND 128),
    CHECK (source_revision > 0),
    CHECK (length(evidence_digest) = 32),
    CHECK (observed_at_unix_seconds > 0 AND observed_at_nanos BETWEEN 0 AND 999999999),
    CHECK (evidence_state BETWEEN 1 AND 2),
    CHECK (updated_at_relationship_revision > 0)
);

CREATE TABLE makosh_data.relationships_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    relationship_id BYTEA NOT NULL,
    relationship_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, relationship_id)
        REFERENCES makosh_data.relationships_records (logical_owner_id, relationship_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 6),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (relationship_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.relationships_outbox (
    logical_owner_id TEXT NOT NULL,
    outbox_sequence BIGSERIAL NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, outbox_sequence),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX relationships_participant_source_idx ON makosh_data.relationships_records
    (logical_owner_id, source_kind, source_public_id, relationship_id);
CREATE INDEX relationships_participant_target_idx ON makosh_data.relationships_records
    (logical_owner_id, target_kind, target_public_id, relationship_id);
CREATE INDEX relationships_evidence_order_idx ON makosh_data.relationships_evidence
    (logical_owner_id, relationship_id, evidence_id);
CREATE INDEX relationships_outbox_pending_idx ON makosh_data.relationships_outbox
    (logical_owner_id, outbox_sequence) WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.relationships_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.relationships_records FORCE ROW LEVEL SECURITY;
CREATE POLICY relationships_records_owner_policy ON makosh_data.relationships_records
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.relationships_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.relationships_evidence FORCE ROW LEVEL SECURITY;
CREATE POLICY relationships_evidence_owner_policy ON makosh_data.relationships_evidence
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.relationships_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.relationships_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY relationships_client_operations_owner_policy ON makosh_data.relationships_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.relationships_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.relationships_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY relationships_outbox_owner_policy ON makosh_data.relationships_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
