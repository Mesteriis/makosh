ALTER TABLE makosh_data.mail_address_book_upsert_inbox
    ADD COLUMN target_contact_snapshot_reference_id BYTEA CHECK (
        target_contact_snapshot_reference_id IS NULL OR
        octet_length(target_contact_snapshot_reference_id) = 16
    ),
    ADD COLUMN target_contact_snapshot_receipt_sha256 BYTEA CHECK (
        target_contact_snapshot_receipt_sha256 IS NULL OR
        octet_length(target_contact_snapshot_receipt_sha256) = 32
    ),
    ADD COLUMN snapshot_custody_recorded_at_unix_seconds BIGINT CHECK (
        snapshot_custody_recorded_at_unix_seconds IS NULL OR
        snapshot_custody_recorded_at_unix_seconds > 0
    ),
    ADD CONSTRAINT mail_address_book_target_snapshot_receipt_complete CHECK (
        (
            target_contact_snapshot_reference_id IS NULL AND
            target_contact_snapshot_receipt_sha256 IS NULL AND
            snapshot_custody_recorded_at_unix_seconds IS NULL
        ) OR (
            target_contact_snapshot_reference_id IS NOT NULL AND
            target_contact_snapshot_receipt_sha256 IS NOT NULL AND
            snapshot_custody_recorded_at_unix_seconds IS NOT NULL
        )
    );
