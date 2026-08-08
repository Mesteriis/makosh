CREATE INDEX attachment_security_scan_jobs_retry_policy_recovery_idx
  ON makosh_data.attachment_security_scan_jobs (
    retry_policy_revision,
    state
  )
  WHERE state = 3
    AND target_blob_reference_id IS NULL
    AND target_blob_receipt_sha256 IS NULL
    AND outbox_message_id IS NULL;
