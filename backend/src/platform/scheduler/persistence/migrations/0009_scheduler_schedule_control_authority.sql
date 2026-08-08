CREATE TABLE makosh_platform.scheduler_schedule_control_authorities (
  schedule_id BYTEA PRIMARY KEY,
  source_module_id TEXT NOT NULL,
  source_owner TEXT NOT NULL,
  job_owner TEXT NOT NULL,
  job_name TEXT NOT NULL,
  job_major INTEGER NOT NULL,
  created_at_unix_ms BIGINT NOT NULL
);
