//! Adds persisted descriptor-declared Scheduler JobKind contract requests.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_scheduler_job_request (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             job_owner TEXT NOT NULL,
             job_name TEXT NOT NULL,
             job_major INTEGER NOT NULL CHECK (job_major BETWEEN 1 AND 4294967295),
             job_revision INTEGER NOT NULL CHECK (job_revision BETWEEN 1 AND 4294967295),
             contract_schema_sha256 BLOB NOT NULL CHECK (length(contract_schema_sha256) = 32),
             PRIMARY KEY (registration_id, capability_id, job_owner, job_name, job_major),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         ) STRICT;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 30 WHERE singleton = 1;",
    )?;
    Ok(())
}
