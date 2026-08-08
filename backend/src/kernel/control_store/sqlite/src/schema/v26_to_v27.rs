//! Adds durable broker-neutral Event Hub stream budgets.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_platform_event_hub_topology (
             singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
             revision INTEGER NOT NULL CHECK (revision >= 1)
         );
         CREATE TABLE makosh_kernel_platform_event_stream_budget (
             envelope_kind INTEGER PRIMARY KEY CHECK (envelope_kind BETWEEN 1 AND 5),
             max_bytes INTEGER NOT NULL CHECK (max_bytes >= 1),
             max_age_millis INTEGER NOT NULL CHECK (max_age_millis >= 1),
             replicas INTEGER NOT NULL CHECK (replicas = 1)
         );
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 27 WHERE singleton = 1;",
    )?;
    Ok(())
}
