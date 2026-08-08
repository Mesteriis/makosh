CREATE TABLE makosh_platform.scheduler_schedules (
  schedule_id BYTEA PRIMARY KEY,
  schedule_revision BIGINT NOT NULL,
  job_owner TEXT NOT NULL,
  job_name TEXT NOT NULL,
  job_major INTEGER NOT NULL,
  contract_name TEXT NOT NULL,
  contract_schema_sha256 BYTEA NOT NULL,
  scope_id TEXT NOT NULL,
  concurrency_key TEXT NOT NULL,
  max_parallelism INTEGER NOT NULL,
  enabled BOOLEAN NOT NULL,
  policy_bytes BYTEA NOT NULL,
  next_due_at_unix_ms BIGINT NOT NULL,
  updated_at_unix_ms BIGINT NOT NULL
);

CREATE TABLE makosh_platform.scheduler_runs (
  run_id BYTEA PRIMARY KEY,
  schedule_id BYTEA NOT NULL,
  schedule_revision BIGINT NOT NULL,
  scheduled_for_unix_ms BIGINT NOT NULL,
  lease_epoch BIGINT NOT NULL,
  lease_expires_at_unix_ms BIGINT NOT NULL,
  state TEXT NOT NULL,
  attempt_count INTEGER NOT NULL,
  dispatch_message_id BYTEA NOT NULL,
  fire_key BYTEA NOT NULL UNIQUE,
  concurrency_key TEXT NOT NULL,
  created_at_unix_ms BIGINT NOT NULL
);

CREATE TABLE makosh_platform.scheduler_concurrency (
  concurrency_key TEXT PRIMARY KEY,
  active_runs INTEGER NOT NULL,
  max_parallelism INTEGER NOT NULL,
  updated_at_unix_ms BIGINT NOT NULL
);
