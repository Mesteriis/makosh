CREATE TABLE makosh_platform.scheduler_schedule_control_inbox (
  command_message_id BYTEA PRIMARY KEY,
  command_envelope_sha256 BYTEA NOT NULL,
  operation_id BYTEA NOT NULL,
  schedule_id BYTEA NOT NULL,
  schedule_revision BIGINT NOT NULL,
  decision TEXT NOT NULL,
  result_message_id BYTEA NOT NULL UNIQUE,
  received_at_unix_ms BIGINT NOT NULL
);

CREATE TABLE makosh_platform.scheduler_schedule_control_results (
  message_id BYTEA PRIMARY KEY,
  command_message_id BYTEA NOT NULL UNIQUE,
  envelope_sha256 BYTEA NOT NULL,
  exact_envelope_bytes BYTEA NOT NULL,
  state TEXT NOT NULL,
  published_stream TEXT,
  published_sequence BIGINT,
  created_at_unix_ms BIGINT NOT NULL
);
