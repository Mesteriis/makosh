ALTER TABLE makosh_data.attachment_security_scan_jobs
  ADD COLUMN retry_policy_revision SMALLINT NOT NULL DEFAULT 1 CHECK (
    retry_policy_revision BETWEEN 1 AND 32767
  );
