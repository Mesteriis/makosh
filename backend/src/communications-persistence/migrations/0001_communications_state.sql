CREATE TABLE makosh_data.communications_event_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32)
);

CREATE TABLE makosh_data.communications_evidence_summaries (
  observation_id BYTEA PRIMARY KEY CHECK (octet_length(observation_id) = 16),
  source_cursor_sha256 BYTEA NOT NULL CHECK (octet_length(source_cursor_sha256) = 32),
  account_cursor_sha256 BYTEA CHECK (
    account_cursor_sha256 IS NULL OR octet_length(account_cursor_sha256) = 32
  ),
  conversation_cursor_sha256 BYTEA CHECK (
    conversation_cursor_sha256 IS NULL OR octet_length(conversation_cursor_sha256) = 32
  ),
  participant_cursor_sha256 BYTEA CHECK (
    participant_cursor_sha256 IS NULL OR octet_length(participant_cursor_sha256) = 32
  ),
  media_cursor_sha256 BYTEA CHECK (
    media_cursor_sha256 IS NULL OR octet_length(media_cursor_sha256) = 32
  ),
  reply_to_source_cursor_sha256 BYTEA CHECK (
    reply_to_source_cursor_sha256 IS NULL
    OR octet_length(reply_to_source_cursor_sha256) = 32
  ),
  forward_origin_source_cursor_sha256 BYTEA CHECK (
    forward_origin_source_cursor_sha256 IS NULL
    OR octet_length(forward_origin_source_cursor_sha256) = 32
  ),
  provider SMALLINT NOT NULL CHECK (provider IN (1, 2, 3, 4, 5, 6)),
  direction SMALLINT NOT NULL CHECK (direction IN (1, 2, 3)),
  evidence_kind SMALLINT NOT NULL CHECK (evidence_kind BETWEEN 1 AND 11),
  body_state SMALLINT NOT NULL CHECK (
    body_state IN (1, 2, 3, 4)
    AND (
      (
        body_state = 4
        AND body_blob_ref IS NOT NULL
        AND body_blob_reference_id IS NOT NULL
        AND body_blob_declared_bytes BETWEEN 1 AND 67108864
        AND body_blob_sha256 IS NOT NULL
        AND octet_length(body_blob_reference_id) = 16
        AND octet_length(body_blob_sha256) = 32
        AND body_admission_failure IS NULL
      )
      OR (
        body_state <> 4
        AND body_blob_ref IS NULL
        AND body_blob_reference_id IS NULL
        AND body_blob_declared_bytes IS NULL
        AND body_blob_sha256 IS NULL
      )
    )
  ),
  body_blob_ref TEXT,
  body_blob_reference_id BYTEA,
  body_blob_declared_bytes BIGINT,
  body_blob_sha256 BYTEA,
  body_admission_failure SMALLINT CHECK (
    body_admission_failure IS NULL OR body_admission_failure IN (1, 2, 3, 4)
  ),
  observed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.communications_domain_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
  created_at_unix_seconds BIGINT NOT NULL,
  published_at_unix_seconds BIGINT
);

CREATE TABLE makosh_data.communications_conversations (
  conversation_id BYTEA PRIMARY KEY CHECK (octet_length(conversation_id) = 16),
  account_cursor_sha256 BYTEA NOT NULL CHECK (octet_length(account_cursor_sha256) = 32),
  conversation_cursor_sha256 BYTEA NOT NULL CHECK (
    octet_length(conversation_cursor_sha256) = 32
  ),
  provider SMALLINT NOT NULL CHECK (provider IN (1, 2, 3, 4, 5, 6)),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_accounts (
  account_id BYTEA PRIMARY KEY CHECK (octet_length(account_id) = 16),
  account_cursor_sha256 BYTEA NOT NULL UNIQUE CHECK (
    octet_length(account_cursor_sha256) = 32
  ),
  provider SMALLINT NOT NULL CHECK (provider IN (1, 2, 3, 4, 5, 6)),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_messages (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  conversation_id BYTEA NOT NULL REFERENCES makosh_data.communications_conversations (
    conversation_id
  ) CHECK (octet_length(conversation_id) = 16),
  source_cursor_sha256 BYTEA NOT NULL UNIQUE CHECK (
    octet_length(source_cursor_sha256) = 32
  ),
  body_state SMALLINT NOT NULL CHECK (body_state IN (1, 2, 3)),
  direction SMALLINT NOT NULL CHECK (direction IN (1, 2, 3)),
  lifecycle_state SMALLINT NOT NULL CHECK (lifecycle_state IN (1, 2)),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_observed_participants (
  participant_id BYTEA PRIMARY KEY CHECK (octet_length(participant_id) = 16),
  conversation_id BYTEA NOT NULL REFERENCES makosh_data.communications_conversations (
    conversation_id
  ) CHECK (octet_length(conversation_id) = 16),
  participant_cursor_sha256 BYTEA NOT NULL CHECK (
    octet_length(participant_cursor_sha256) = 32
  ),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_attachment_anchors (
  attachment_anchor_id BYTEA PRIMARY KEY CHECK (
    octet_length(attachment_anchor_id) = 16
  ),
  message_id BYTEA NOT NULL REFERENCES makosh_data.communications_messages (
    message_id
  ) CHECK (octet_length(message_id) = 16),
  media_cursor_sha256 BYTEA NOT NULL CHECK (octet_length(media_cursor_sha256) = 32),
  anchor_state SMALLINT NOT NULL CHECK (anchor_state BETWEEN 1 AND 6),
  attachment_filename TEXT,
  attachment_media_type TEXT CHECK (
    (
      attachment_media_type IS NULL
      AND attachment_declared_bytes IS NULL
      AND attachment_disposition IS NULL
      AND attachment_sha256 IS NULL
    )
    OR (
      attachment_media_type IS NOT NULL
      AND attachment_declared_bytes IS NOT NULL
      AND attachment_declared_bytes >= 0
      AND attachment_disposition IN (1, 2, 3)
      AND (
        attachment_sha256 IS NULL
        OR octet_length(attachment_sha256) = 32
      )
    )
  ),
  attachment_declared_bytes BIGINT,
  attachment_sha256 BYTEA,
  attachment_disposition SMALLINT,
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_message_references (
  reference_id BYTEA PRIMARY KEY CHECK (octet_length(reference_id) = 32),
  source_message_id BYTEA NOT NULL REFERENCES makosh_data.communications_messages (
    message_id
  ) CHECK (octet_length(source_message_id) = 16),
  reference_kind SMALLINT NOT NULL CHECK (reference_kind IN (1, 2)),
  target_source_cursor_sha256 BYTEA NOT NULL CHECK (
    octet_length(target_source_cursor_sha256) = 32
  ),
  observed_at_unix_seconds BIGINT NOT NULL,
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16)
);
