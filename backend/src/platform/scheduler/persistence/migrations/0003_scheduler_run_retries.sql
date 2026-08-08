CREATE TABLE makosh_platform.scheduler_run_retries (
  run_id BYTEA PRIMARY KEY,
  retry_max_attempts INTEGER NOT NULL,
  retry_base_backoff_millis BIGINT NOT NULL,
  next_attempt_at_unix_ms BIGINT
);
