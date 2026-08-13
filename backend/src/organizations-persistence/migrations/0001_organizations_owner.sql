CREATE TABLE makosh_data.organizations_records (
    logical_owner_id TEXT NOT NULL,
    organization_id BYTEA NOT NULL,
    display_name TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    description TEXT NOT NULL,
    website TEXT NOT NULL,
    industry TEXT NOT NULL,
    country_code TEXT NOT NULL,
    organization_state SMALLINT NOT NULL,
    organization_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, organization_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(organization_id) = 16),
    CHECK (char_length(display_name) BETWEEN 1 AND 240),
    CHECK (char_length(legal_name) <= 320),
    CHECK (char_length(description) <= 8000),
    CHECK (char_length(website) <= 512),
    CHECK (char_length(industry) <= 160),
    CHECK (country_code = '' OR (length(country_code) = 2 AND country_code = upper(country_code))),
    CHECK (organization_state BETWEEN 1 AND 2),
    CHECK (organization_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.organizations_sources (
    logical_owner_id TEXT NOT NULL,
    organization_id BYTEA NOT NULL,
    source_id BYTEA NOT NULL,
    source_owner_id TEXT NOT NULL,
    source_record_id TEXT NOT NULL,
    source_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    source_state SMALLINT NOT NULL,
    updated_at_organization_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, organization_id, source_id),
    UNIQUE (logical_owner_id, organization_id, source_owner_id, source_record_id),
    FOREIGN KEY (logical_owner_id, organization_id)
        REFERENCES makosh_data.organizations_records (logical_owner_id, organization_id) ON DELETE CASCADE,
    CHECK (length(source_id) = 16),
    CHECK (length(source_owner_id) BETWEEN 1 AND 256),
    CHECK (length(source_record_id) BETWEEN 1 AND 256),
    CHECK (source_revision > 0),
    CHECK (length(evidence_digest) = 32),
    CHECK (source_state BETWEEN 1 AND 2),
    CHECK (updated_at_organization_revision > 0)
);

CREATE TABLE makosh_data.organizations_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    organization_id BYTEA NOT NULL,
    organization_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, organization_id)
        REFERENCES makosh_data.organizations_records (logical_owner_id, organization_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 5),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (organization_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.organizations_outbox (
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

CREATE INDEX organizations_order_idx
ON makosh_data.organizations_records (logical_owner_id, organization_id);
CREATE INDEX organization_sources_order_idx
ON makosh_data.organizations_sources (logical_owner_id, organization_id, source_id);
CREATE INDEX organization_outbox_pending_idx
ON makosh_data.organizations_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.organizations_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.organizations_records FORCE ROW LEVEL SECURITY;
CREATE POLICY organizations_records_owner_policy ON makosh_data.organizations_records
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.organizations_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.organizations_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY organizations_sources_owner_policy ON makosh_data.organizations_sources
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.organizations_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.organizations_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY organizations_client_operations_owner_policy ON makosh_data.organizations_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.organizations_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.organizations_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY organizations_outbox_owner_policy ON makosh_data.organizations_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
