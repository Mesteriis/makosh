//! Explicit database hooks for live owner-persistence conformance.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{TelegramDurablePersistence, TelegramDurablePersistenceError};

pub struct TelegramPersistenceConformanceV1;

impl TelegramPersistenceConformanceV1 {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_id: &str,
    ) -> Result<TelegramDurablePersistence, TelegramDurablePersistenceError> {
        if host.trim().is_empty()
            || port == 0
            || username.trim().is_empty()
            || password.is_empty()
            || database_id.trim().is_empty()
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database_id);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(TelegramDurablePersistence::new(pool))
    }

    pub async fn reopen_publish_before_mark_window(
        persistence: &TelegramDurablePersistence,
        message_id: &[u8; 16],
    ) -> Result<bool, TelegramDurablePersistenceError> {
        let result = sqlx::query(
            "UPDATE makosh_data.telegram_communications_outbox \
             SET published_at_unix_seconds = NULL \
             WHERE message_id = $1",
        )
        .bind(message_id.as_slice())
        .execute(&persistence.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(result.rows_affected() == 1)
    }
}
