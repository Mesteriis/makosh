CREATE TABLE makosh_platform.scheduler_pending_fires (
  fire_key BYTEA PRIMARY KEY,
  schedule_id BYTEA NOT NULL,
  schedule_revision BIGINT NOT NULL,
  scheduled_for_unix_ms BIGINT NOT NULL,
  concurrency_key TEXT NOT NULL,
  recorded_at_unix_ms BIGINT NOT NULL
);
