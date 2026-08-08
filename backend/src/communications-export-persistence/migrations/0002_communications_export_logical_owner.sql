ALTER TABLE makosh_data.communications_export_jobs
  ADD COLUMN logical_owner_id TEXT CHECK (
    logical_owner_id IS NULL
    OR (
      char_length(logical_owner_id) BETWEEN 1 AND 128
      AND octet_length(logical_owner_id) = char_length(logical_owner_id)
    )
  );

CREATE INDEX communications_export_jobs_owner_pending_idx
  ON makosh_data.communications_export_jobs (
    logical_owner_id,
    updated_at_unix_seconds,
    export_id
  )
  WHERE logical_owner_id IS NOT NULL AND state IN (1, 2);
