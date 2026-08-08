//! Disposable PostgreSQL boundary for Mail address-book persistence conformance only.

use std::str::FromStr;

use sqlx::{
    Executor,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1, MAIL_ADDRESS_BOOK_PROVIDER_PAGE_SCHEMA_V1,
    MAIL_ADDRESS_BOOK_SCHEMA_V1, MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
};

pub struct MailAddressBookPersistenceConformanceV1;

impl MailAddressBookPersistenceConformanceV1 {
    pub async fn connect_url(
        database_url: &str,
    ) -> Result<MailAddressBookPersistenceV1, MailAddressBookPersistenceErrorV1> {
        if database_url.trim().is_empty() {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let options = PgConnectOptions::from_str(database_url)
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|_| MailAddressBookPersistenceErrorV1::StorageUnavailable)?;
        Ok(MailAddressBookPersistenceV1::from_owner_local_pool(pool))
    }

    pub async fn install_schema(
        persistence: &MailAddressBookPersistenceV1,
    ) -> Result<(), MailAddressBookPersistenceErrorV1> {
        persistence
            .pool
            .execute("DROP SCHEMA IF EXISTS makosh_data CASCADE")
            .await
            .map_err(storage_error)?;
        persistence
            .pool
            .execute("CREATE SCHEMA makosh_data")
            .await
            .map_err(storage_error)?;
        persistence
            .pool
            .execute(
                "CREATE TABLE makosh_data.mail_gmail_oauth_credential_bindings (
                    account_id TEXT PRIMARY KEY
                )",
            )
            .await
            .map_err(storage_error)?;
        for migration in [
            MAIL_ADDRESS_BOOK_SCHEMA_V1,
            MAIL_ADDRESS_BOOK_CUSTODY_SCHEMA_V1,
            MAIL_ADDRESS_BOOK_PROVIDER_PAGE_SCHEMA_V1,
        ] {
            let sql = std::str::from_utf8(migration)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.to_owned()))
                .execute(&persistence.pool)
                .await
                .map_err(storage_error)?;
        }
        Ok(())
    }
}

fn storage_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::StorageUnavailable
}
