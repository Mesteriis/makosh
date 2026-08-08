use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_vault_purpose_request_next (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             purpose_id TEXT NOT NULL,
             requested_lease_ttl_seconds INTEGER NOT NULL CHECK (requested_lease_ttl_seconds BETWEEN 1 AND 65535),
             secret_class INTEGER NOT NULL CHECK (secret_class BETWEEN 1 AND 6),
             action INTEGER NOT NULL CHECK (action BETWEEN 1 AND 7),
             target_scope INTEGER NOT NULL CHECK (target_scope IN (1, 2)),
             key_schema_revision INTEGER NOT NULL CHECK (key_schema_revision >= 0),
             PRIMARY KEY (registration_id, capability_id, purpose_id, secret_class, action, target_scope, key_schema_revision),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         );
         INSERT INTO makosh_kernel_module_vault_purpose_request_next
             (registration_id, capability_id, purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope, key_schema_revision)
             SELECT registration_id, capability_id, purpose_id, requested_lease_ttl_seconds, secret_class, action, target_scope, 0
             FROM makosh_kernel_module_vault_purpose_request;
         DROP TABLE makosh_kernel_module_vault_purpose_request;
         ALTER TABLE makosh_kernel_module_vault_purpose_request_next RENAME TO makosh_kernel_module_vault_purpose_request;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 38 WHERE singleton = 1;",
    )?;
    Ok(())
}
