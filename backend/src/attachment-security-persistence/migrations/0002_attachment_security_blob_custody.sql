ALTER TABLE makosh_data.attachment_security_scan_candidates
  ADD COLUMN custody_transfer_source_proof BYTEA NOT NULL CHECK (
    octet_length(custody_transfer_source_proof) BETWEEN 1 AND 2048
  );

ALTER TABLE makosh_data.attachment_security_scan_jobs
  ADD COLUMN target_blob_reference_id BYTEA CHECK (
    target_blob_reference_id IS NULL
    OR octet_length(target_blob_reference_id) = 16
  ),
  ADD COLUMN target_blob_receipt_sha256 BYTEA CHECK (
    target_blob_receipt_sha256 IS NULL
    OR octet_length(target_blob_receipt_sha256) = 32
  ),
  ADD CONSTRAINT attachment_security_target_blob_receipt_complete CHECK (
    (target_blob_reference_id IS NULL AND target_blob_receipt_sha256 IS NULL)
    OR (
      target_blob_reference_id IS NOT NULL
      AND target_blob_receipt_sha256 IS NOT NULL
    )
  );
