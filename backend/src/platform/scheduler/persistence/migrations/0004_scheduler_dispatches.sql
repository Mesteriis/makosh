CREATE TABLE makosh_platform.scheduler_dispatches (
  run_id BYTEA NOT NULL,
  lease_epoch BIGINT NOT NULL,
  message_id BYTEA PRIMARY KEY,
  envelope_sha256 BYTEA NOT NULL,
  exact_envelope_bytes BYTEA NOT NULL,
  state TEXT NOT NULL,
  published_stream TEXT,
  published_sequence BIGINT,
  created_at_unix_ms BIGINT NOT NULL
);
