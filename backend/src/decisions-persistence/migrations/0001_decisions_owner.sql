CREATE TABLE makosh_data.decisions_records (
    logical_owner_id TEXT NOT NULL,
    decision_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    question TEXT NOT NULL,
    rationale TEXT NOT NULL DEFAULT '',
    decision_state SMALLINT NOT NULL,
    selected_alternative_id BYTEA,
    superseded_by_decision_id BYTEA,
    decision_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, decision_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(decision_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 240),
    CHECK (char_length(question) BETWEEN 1 AND 4000),
    CHECK (char_length(rationale) <= 8000),
    CHECK (decision_state BETWEEN 1 AND 4),
    CHECK (selected_alternative_id IS NULL OR length(selected_alternative_id) = 16),
    CHECK (superseded_by_decision_id IS NULL OR length(superseded_by_decision_id) = 16),
    CHECK (decision_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.decisions_alternatives (
    logical_owner_id TEXT NOT NULL,
    decision_id BYTEA NOT NULL,
    alternative_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    alternative_state SMALLINT NOT NULL,
    alternative_revision BIGINT NOT NULL,
    updated_at_decision_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, decision_id, alternative_id),
    FOREIGN KEY (logical_owner_id, decision_id)
        REFERENCES makosh_data.decisions_records (logical_owner_id, decision_id) ON DELETE CASCADE,
    CHECK (length(alternative_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 240),
    CHECK (char_length(description) <= 8000),
    CHECK (alternative_state BETWEEN 1 AND 3),
    CHECK (alternative_revision > 0),
    CHECK (updated_at_decision_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE UNIQUE INDEX decisions_one_selected_alternative_idx
ON makosh_data.decisions_alternatives (logical_owner_id, decision_id)
WHERE alternative_state = 2;

CREATE TABLE makosh_data.decisions_evidence_links (
    logical_owner_id TEXT NOT NULL,
    decision_id BYTEA NOT NULL,
    evidence_link_id BYTEA NOT NULL,
    evidence_owner_id TEXT NOT NULL,
    evidence_record_id BYTEA NOT NULL,
    evidence_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, decision_id, evidence_link_id),
    UNIQUE (logical_owner_id, decision_id, evidence_owner_id, evidence_record_id, evidence_revision),
    FOREIGN KEY (logical_owner_id, decision_id)
        REFERENCES makosh_data.decisions_records (logical_owner_id, decision_id) ON DELETE CASCADE,
    CHECK (length(evidence_link_id) = 16),
    CHECK (length(evidence_owner_id) BETWEEN 1 AND 128),
    CHECK (length(evidence_record_id) = 16),
    CHECK (evidence_revision > 0),
    CHECK (length(evidence_digest) = 32)
);

CREATE TABLE makosh_data.decisions_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    decision_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, decision_id)
        REFERENCES makosh_data.decisions_records (logical_owner_id, decision_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 10),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (decision_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.decisions_outbox (
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

CREATE INDEX decisions_records_order_idx
ON makosh_data.decisions_records (logical_owner_id, decision_id);
CREATE INDEX decisions_alternatives_order_idx
ON makosh_data.decisions_alternatives (logical_owner_id, decision_id, alternative_id);
CREATE INDEX decisions_evidence_order_idx
ON makosh_data.decisions_evidence_links (logical_owner_id, decision_id, evidence_link_id);
CREATE INDEX decisions_outbox_pending_idx
ON makosh_data.decisions_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.decisions_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.decisions_records FORCE ROW LEVEL SECURITY;
CREATE POLICY decisions_records_owner_policy ON makosh_data.decisions_records
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.decisions_alternatives ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.decisions_alternatives FORCE ROW LEVEL SECURITY;
CREATE POLICY decisions_alternatives_owner_policy ON makosh_data.decisions_alternatives
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.decisions_evidence_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.decisions_evidence_links FORCE ROW LEVEL SECURITY;
CREATE POLICY decisions_evidence_links_owner_policy ON makosh_data.decisions_evidence_links
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.decisions_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.decisions_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY decisions_client_operations_owner_policy ON makosh_data.decisions_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.decisions_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.decisions_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY decisions_outbox_owner_policy ON makosh_data.decisions_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
