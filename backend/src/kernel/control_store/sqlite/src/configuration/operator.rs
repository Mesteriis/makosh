//! Kernel-owned operator settings that must be available before other services.

use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn developer_mode_enabled(&self) -> Result<bool, StoreError> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    "SELECT developer_mode_enabled
                     FROM makosh_kernel_operator_settings WHERE singleton = 1",
                    [],
                    |row| row.get::<_, bool>(0),
                )
                .optional()?
                .ok_or(StoreError::MissingMetadata)
        })
    }

    pub fn set_developer_mode_enabled(&self, enabled: bool) -> Result<(), StoreError> {
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "UPDATE makosh_kernel_operator_settings
                 SET developer_mode_enabled = ?1 WHERE singleton = 1",
                params![enabled],
            )?;
            (changed == 1)
                .then_some(())
                .ok_or(StoreError::MissingMetadata)
        })
    }
}
