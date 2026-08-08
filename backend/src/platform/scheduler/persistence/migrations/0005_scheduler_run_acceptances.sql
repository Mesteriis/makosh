CREATE TABLE makosh_platform.scheduler_run_acceptances (
  command_message_id BYTEA PRIMARY KEY,
  run_id BYTEA NOT NULL,
  lease_epoch BIGINT NOT NULL,
  observed_at_unix_ms BIGINT NOT NULL
);
