CREATE TABLE makosh_data.communications_saved_query_definitions (
  saved_search_id BYTEA PRIMARY KEY CHECK (octet_length(saved_search_id) = 16),
  name TEXT NOT NULL CHECK (
    octet_length(name) BETWEEN 1 AND 128
    AND name !~ '[[:cntrl:]]'
  ),
  description TEXT CHECK (
    description IS NULL
    OR (
      octet_length(description) BETWEEN 1 AND 512
      AND description !~ '[[:cntrl:]]'
    )
  ),
  account_id BYTEA REFERENCES makosh_data.communications_accounts (account_id)
    CHECK (account_id IS NULL OR octet_length(account_id) = 16),
  token_count SMALLINT NOT NULL CHECK (token_count BETWEEN 1 AND 16),
  key_schema_revision INTEGER NOT NULL CHECK (key_schema_revision > 0),
  lifecycle_state SMALLINT NOT NULL CHECK (lifecycle_state IN (1, 2)),
  revision BIGINT NOT NULL CHECK (revision > 0),
  created_at_unix_seconds BIGINT NOT NULL,
  updated_at_unix_seconds BIGINT NOT NULL CHECK (
    updated_at_unix_seconds >= created_at_unix_seconds
  )
);

CREATE TABLE makosh_data.communications_saved_query_token_digests (
  saved_search_id BYTEA NOT NULL REFERENCES makosh_data.communications_saved_query_definitions (
    saved_search_id
  ) ON DELETE CASCADE CHECK (octet_length(saved_search_id) = 16),
  position SMALLINT NOT NULL CHECK (position BETWEEN 0 AND 15),
  token_digest BYTEA NOT NULL CHECK (octet_length(token_digest) = 32),
  PRIMARY KEY (saved_search_id, token_digest),
  UNIQUE (saved_search_id, position)
);

CREATE TABLE makosh_data.communications_saved_query_audit (
  saved_search_id BYTEA NOT NULL CHECK (octet_length(saved_search_id) = 16),
  revision BIGINT NOT NULL CHECK (revision > 0),
  change_kind SMALLINT NOT NULL CHECK (change_kind IN (1, 2, 3)),
  definition_sha256 BYTEA NOT NULL CHECK (octet_length(definition_sha256) = 32),
  changed_at_unix_seconds BIGINT NOT NULL,
  PRIMARY KEY (saved_search_id, revision)
);

CREATE INDEX communications_saved_query_definitions_list_v1
  ON makosh_data.communications_saved_query_definitions (
    updated_at_unix_seconds DESC,
    saved_search_id ASC
  )
  WHERE lifecycle_state = 1;
