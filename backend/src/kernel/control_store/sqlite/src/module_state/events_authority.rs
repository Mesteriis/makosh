//! SQLite persistence for managed Events Authority public configuration.

use makosh_kernel_control_store::PlatformEventsAuthorityConfigurationV1;
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn record_platform_events_authority_configuration(
        &self,
        configuration: &PlatformEventsAuthorityConfigurationV1,
    ) -> Result<(), StoreError> {
        valid_configuration(configuration)
            .then_some(())
            .ok_or(StoreError::InvalidPlatformEventsAuthorityConfiguration)?;
        let configuration = configuration.clone();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_platform_events_authority_configuration (singleton, revision, account_public_key, signer_credential_revision) VALUES (1, ?1, ?2, ?3) ON CONFLICT(singleton) DO UPDATE SET revision=excluded.revision, account_public_key=excluded.account_public_key, signer_credential_revision=excluded.signer_credential_revision WHERE excluded.revision = makosh_kernel_platform_events_authority_configuration.revision + 1 AND excluded.signer_credential_revision >= makosh_kernel_platform_events_authority_configuration.signer_credential_revision",
                params![as_sql(configuration.revision())?, configuration.account_public_key(), as_sql(configuration.signer_credential_revision())?],
            )?;
            (changed == 1)
                .then_some(())
                .ok_or(StoreError::PlatformEventsAuthorityConfigurationRevisionConflict)
        })
    }

    pub fn platform_events_authority_configuration(
        &self,
    ) -> Result<Option<PlatformEventsAuthorityConfigurationV1>, StoreError> {
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT revision, account_public_key, signer_credential_revision FROM makosh_kernel_platform_events_authority_configuration WHERE singleton=1",
                    [],
                    |row| Ok(PlatformEventsAuthorityConfigurationV1::new(as_u64(row.get(0)?, 0)?, row.get::<_, String>(1)?, as_u64(row.get(2)?, 2)?)),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
}

fn valid_configuration(value: &PlatformEventsAuthorityConfigurationV1) -> bool {
    value.revision() > 0
        && valid_account_key(value.account_public_key())
        && value.signer_credential_revision() > 0
}

fn valid_account_key(value: &str) -> bool {
    value.len() == 56
        && value.starts_with('A')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformEventsAuthorityConfiguration)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
