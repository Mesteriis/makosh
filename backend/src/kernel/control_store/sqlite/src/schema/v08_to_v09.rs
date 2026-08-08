use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_settings_schema_binding
            ADD COLUMN apply_state TEXT NOT NULL DEFAULT 'current'
            CHECK (apply_state IN ('current', 'pending_validation', 'pending_apply', 'applying', 'awaiting_external_restart', 'blocked_config'));
        ALTER TABLE makosh_kernel_settings_schema_binding ADD COLUMN sanitized_reason_code TEXT;
        UPDATE makosh_kernel_settings_schema_binding
            SET apply_state = CASE
                WHEN desired_revision = effective_revision THEN 'current'
                ELSE 'pending_validation'
            END;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 9 WHERE singleton = 1;",
    )?;
    Ok(())
}
