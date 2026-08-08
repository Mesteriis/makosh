CREATE TABLE makosh_data.attachment_text_extraction_translation_source_inbox (
    logical_owner_id TEXT NOT NULL,
    request_message_id BYTEA NOT NULL,
    request_envelope_sha256 BYTEA NOT NULL,
    request_id BYTEA NOT NULL,
    translation_run_id BYTEA NOT NULL,
    source_extraction_run_id BYTEA NOT NULL,
    expected_source_revision BIGINT NOT NULL,
    result_message_id BYTEA NOT NULL,
    result_envelope_sha256 BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, request_message_id),
    UNIQUE (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_message_id) = 16),
    CHECK (length(request_envelope_sha256) = 32),
    CHECK (length(request_id) = 16),
    CHECK (length(translation_run_id) = 16),
    CHECK (length(source_extraction_run_id) = 16),
    CHECK (expected_source_revision > 0),
    CHECK (length(result_message_id) = 16),
    CHECK (length(result_envelope_sha256) = 32),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_text_extraction_translation_source_outbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    request_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    published_at_unix_millis BIGINT,
    created_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    UNIQUE (logical_owner_id, request_message_id),
    FOREIGN KEY (logical_owner_id, request_message_id)
        REFERENCES makosh_data.attachment_text_extraction_translation_source_inbox (
            logical_owner_id,
            request_message_id
        ),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id) = 16),
    CHECK (length(request_message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(exact_envelope_bytes) BETWEEN 1 AND 8192),
    CHECK (
        published_at_unix_millis IS NULL
        OR published_at_unix_millis >= created_at_unix_millis
    ),
    CHECK (created_at_unix_millis > 0)
);

CREATE INDEX attachment_text_extraction_translation_source_outbox_pending_idx
ON makosh_data.attachment_text_extraction_translation_source_outbox (
    logical_owner_id,
    published_at_unix_millis,
    created_at_unix_millis,
    result_message_id
);
