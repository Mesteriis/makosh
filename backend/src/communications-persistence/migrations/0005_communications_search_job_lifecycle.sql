ALTER TABLE makosh_data.communications_derived_index_jobs
  ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
  ADD COLUMN claimed_by TEXT,
  ADD COLUMN lease_expires_at_unix_seconds BIGINT,
  ADD COLUMN outcome SMALLINT CHECK (outcome IN (1, 2)),
  ADD COLUMN failure_code SMALLINT CHECK (failure_code IN (1, 2, 3, 4, 5, 6));

ALTER TABLE makosh_data.communications_derived_index_jobs
  ADD CONSTRAINT communications_derived_index_jobs_lifecycle_shape CHECK (
    (completed_at_unix_seconds IS NULL AND outcome IS NULL AND failure_code IS NULL)
    OR (completed_at_unix_seconds IS NOT NULL AND outcome = 1 AND failure_code IS NULL)
    OR (completed_at_unix_seconds IS NOT NULL AND outcome = 2 AND failure_code IS NOT NULL)
  );

CREATE INDEX communications_derived_index_jobs_claimable
  ON makosh_data.communications_derived_index_jobs (lease_expires_at_unix_seconds, created_at_unix_seconds, job_id)
  WHERE completed_at_unix_seconds IS NULL;
