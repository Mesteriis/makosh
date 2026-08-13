CREATE TABLE makosh_data.documents_records (
    logical_owner_id TEXT NOT NULL,
    document_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    media_type TEXT NOT NULL,
    original_file_name TEXT NOT NULL,
    declared_size BIGINT NOT NULL,
    content_sha256 BYTEA NOT NULL,
    document_state SMALLINT NOT NULL,
    custody_state SMALLINT NOT NULL,
    blob_reference_id BYTEA,
    custody_updated_document_revision BIGINT,
    document_revision BIGINT NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, document_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(document_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 512),
    CHECK (char_length(description) <= 16384),
    CHECK (char_length(media_type) BETWEEN 1 AND 256),
    CHECK (char_length(original_file_name) BETWEEN 1 AND 1024),
    CHECK (declared_size BETWEEN 1 AND 1099511627776),
    CHECK (length(content_sha256) = 32),
    CHECK (document_state BETWEEN 1 AND 2),
    CHECK (custody_state BETWEEN 1 AND 3),
    CHECK ((custody_state = 1 AND blob_reference_id IS NULL AND custody_updated_document_revision IS NULL) OR
           (custody_state IN (2,3) AND length(blob_reference_id) = 16 AND custody_updated_document_revision > 0)),
    CHECK (document_revision > 0),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE TABLE makosh_data.documents_sources (
    logical_owner_id TEXT NOT NULL,
    document_id BYTEA NOT NULL,
    source_id BYTEA NOT NULL,
    source_owner_id TEXT NOT NULL,
    source_record_id TEXT NOT NULL,
    source_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    source_state SMALLINT NOT NULL,
    updated_at_document_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, document_id, source_id),
    UNIQUE (logical_owner_id, document_id, source_owner_id, source_record_id),
    FOREIGN KEY (logical_owner_id, document_id)
        REFERENCES makosh_data.documents_records (logical_owner_id, document_id) ON DELETE CASCADE,
    CHECK (length(source_id) = 16),
    CHECK (length(source_owner_id) BETWEEN 1 AND 128),
    CHECK (length(source_record_id) BETWEEN 1 AND 128),
    CHECK (source_revision > 0),
    CHECK (length(evidence_digest) = 32),
    CHECK (source_state BETWEEN 1 AND 2),
    CHECK (updated_at_document_revision > 0)
);

CREATE TABLE makosh_data.documents_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    document_id BYTEA NOT NULL,
    expected_document_revision BIGINT,
    response_sha256 BYTEA,
    response_bytes BYTEA,
    operation_state SMALLINT NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    completed_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, document_id)
        REFERENCES makosh_data.documents_records (logical_owner_id, document_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 7),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (expected_document_revision IS NULL OR expected_document_revision > 0),
    CHECK (operation_state BETWEEN 1 AND 2),
    CHECK ((operation_state = 1 AND response_sha256 IS NULL AND response_bytes IS NULL AND completed_at_unix_millis IS NULL) OR
           (operation_state = 2 AND length(response_sha256) = 32 AND length(response_bytes) BETWEEN 1 AND 65536 AND completed_at_unix_millis >= received_at_unix_millis)),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.documents_blob_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    document_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    expected_document_revision BIGINT NOT NULL,
    blob_reference_id BYTEA NOT NULL,
    declared_size BIGINT,
    content_sha256 BYTEA,
    changed_at_unix_millis BIGINT NOT NULL,
    custody_source_proof BYTEA NOT NULL,
    source_evidence_id BYTEA,
    source_evidence_envelope_sha256 BYTEA,
    provider_request_sha256 BYTEA NOT NULL,
    provider_request_bytes BYTEA NOT NULL,
    provider_receipt_sha256 BYTEA,
    provider_receipt_bytes BYTEA,
    resolved_blob_reference_id BYTEA,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, operation_id)
        REFERENCES makosh_data.documents_client_operations (logical_owner_id, operation_id) ON DELETE CASCADE,
    CHECK (operation_kind IN (4,5)),
    CHECK (length(blob_reference_id) = 16),
    CHECK ((operation_kind = 4 AND declared_size > 0 AND length(content_sha256) = 32) OR
           (operation_kind = 5 AND declared_size IS NULL AND content_sha256 IS NULL)),
    CHECK (changed_at_unix_millis > 0),
    CHECK (length(custody_source_proof) BETWEEN 1 AND 2048),
    CHECK ((operation_kind = 4 AND length(source_evidence_id) = 16 AND length(source_evidence_envelope_sha256) = 32) OR
           (operation_kind = 5 AND source_evidence_id IS NULL AND source_evidence_envelope_sha256 IS NULL)),
    CHECK (length(provider_request_sha256) = 32),
    CHECK (length(provider_request_bytes) BETWEEN 1 AND 65536),
    CHECK ((provider_receipt_sha256 IS NULL AND provider_receipt_bytes IS NULL) OR
           (length(provider_receipt_sha256) = 32 AND length(provider_receipt_bytes) BETWEEN 1 AND 65536)),
    CHECK (resolved_blob_reference_id IS NULL OR length(resolved_blob_reference_id) = 16)
);

CREATE TABLE makosh_data.documents_outbox (
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

CREATE INDEX documents_order_idx ON makosh_data.documents_records (logical_owner_id, document_id);
CREATE INDEX documents_sources_order_idx ON makosh_data.documents_sources (logical_owner_id, document_id, source_id);
CREATE INDEX documents_outbox_pending_idx ON makosh_data.documents_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.documents_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.documents_records FORCE ROW LEVEL SECURITY;
CREATE POLICY documents_records_owner_policy ON makosh_data.documents_records
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.documents_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.documents_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY documents_sources_owner_policy ON makosh_data.documents_sources
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.documents_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.documents_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY documents_client_operations_owner_policy ON makosh_data.documents_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.documents_blob_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.documents_blob_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY documents_blob_operations_owner_policy ON makosh_data.documents_blob_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.documents_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.documents_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY documents_outbox_owner_policy ON makosh_data.documents_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
