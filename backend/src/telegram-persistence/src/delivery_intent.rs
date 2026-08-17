//! Telegram-owned delivery-intent route locator projection.

use makosh_communications_ingress::{
    ProviderProvenanceV1, account_source_cursor_v1, conversation_source_cursor_v1,
    scoped_record_source_cursor_v1,
};
use makosh_telegram_api::TelegramMessageProjection;
use sqlx::{Postgres, Transaction};

use crate::TelegramDurablePersistenceError;

pub const TELEGRAM_SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_route_accounts (
    account_cursor BYTEA PRIMARY KEY,
    account_id TEXT NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(account_cursor) = 32),
    CHECK (length(account_id) BETWEEN 1 AND 256)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_route_conversations (
    conversation_cursor BYTEA PRIMARY KEY,
    account_cursor BYTEA NOT NULL,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    UNIQUE (account_id, provider_chat_id),
    CHECK (octet_length(conversation_cursor) = 32),
    CHECK (octet_length(account_cursor) = 32),
    CHECK (length(account_id) BETWEEN 1 AND 256),
    CHECK (length(provider_chat_id) BETWEEN 1 AND 512)
);

CREATE INDEX IF NOT EXISTS telegram_delivery_route_conversations_account_idx
    ON makosh_data.telegram_delivery_route_conversations
        (account_cursor, updated_at_unix_seconds DESC);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_route_messages (
    source_cursor BYTEA PRIMARY KEY,
    account_cursor BYTEA NOT NULL,
    conversation_cursor BYTEA NOT NULL,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    UNIQUE (account_id, provider_chat_id, provider_message_id),
    CHECK (octet_length(source_cursor) = 32),
    CHECK (octet_length(account_cursor) = 32),
    CHECK (octet_length(conversation_cursor) = 32),
    CHECK (length(account_id) BETWEEN 1 AND 256),
    CHECK (length(provider_chat_id) BETWEEN 1 AND 512),
    CHECK (length(provider_message_id) BETWEEN 1 AND 512)
);

CREATE INDEX IF NOT EXISTS telegram_delivery_route_messages_conversation_idx
    ON makosh_data.telegram_delivery_route_messages
        (conversation_cursor, updated_at_unix_seconds DESC);
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramDeliveryRouteLocatorV1 {
    pub account_cursor: [u8; 32],
    pub conversation_cursor: [u8; 32],
    pub source_cursor: [u8; 32],
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
}

impl TelegramDeliveryRouteLocatorV1 {
    pub fn from_message(
        message: &TelegramMessageProjection,
    ) -> Result<Self, TelegramDurablePersistenceError> {
        let account_id = message.account_id.to_string();
        let source_id = format!(
            "telegram:{account_id}:{}:{}",
            message.provider_chat_id, message.provider_message_id
        );
        Ok(Self {
            account_cursor: account_source_cursor_v1(ProviderProvenanceV1::Telegram, &account_id)
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            conversation_cursor: conversation_source_cursor_v1(
                ProviderProvenanceV1::Telegram,
                &account_id,
                &message.provider_chat_id,
            )
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            source_cursor: scoped_record_source_cursor_v1(
                ProviderProvenanceV1::Telegram,
                &account_id,
                &source_id,
            )
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            account_id,
            provider_chat_id: message.provider_chat_id.clone(),
            provider_message_id: message.provider_message_id.clone(),
        })
    }
}

pub(crate) async fn upsert_delivery_route_locator(
    transaction: &mut Transaction<'_, Postgres>,
    locator: &TelegramDeliveryRouteLocatorV1,
    updated_at_unix_seconds: i64,
) -> Result<(), TelegramDurablePersistenceError> {
    if !valid_locator(locator) || updated_at_unix_seconds <= 0 {
        return Err(TelegramDurablePersistenceError::InvalidRow);
    }
    let account = sqlx::query(
        "INSERT INTO makosh_data.telegram_delivery_route_accounts
            (account_cursor, account_id, active, updated_at_unix_seconds)
         VALUES ($1, $2, TRUE, $3)
         ON CONFLICT (account_cursor) DO UPDATE SET
            active = TRUE,
            updated_at_unix_seconds = GREATEST(
                makosh_data.telegram_delivery_route_accounts.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.telegram_delivery_route_accounts.account_id = EXCLUDED.account_id",
    )
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.account_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramDurablePersistenceError::Database)?;
    if account.rows_affected() != 1 {
        return Err(TelegramDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }

    let conversation = sqlx::query(
        "INSERT INTO makosh_data.telegram_delivery_route_conversations
            (conversation_cursor, account_cursor, account_id, provider_chat_id,
             updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (conversation_cursor) DO UPDATE SET
            updated_at_unix_seconds = GREATEST(
                makosh_data.telegram_delivery_route_conversations.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.telegram_delivery_route_conversations.account_cursor =
                   EXCLUDED.account_cursor
           AND makosh_data.telegram_delivery_route_conversations.account_id =
                   EXCLUDED.account_id
           AND makosh_data.telegram_delivery_route_conversations.provider_chat_id =
                   EXCLUDED.provider_chat_id",
    )
    .bind(locator.conversation_cursor.as_slice())
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.account_id)
    .bind(&locator.provider_chat_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramDurablePersistenceError::Database)?;
    if conversation.rows_affected() != 1 {
        return Err(TelegramDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }

    let message = sqlx::query(
        "INSERT INTO makosh_data.telegram_delivery_route_messages
            (source_cursor, account_cursor, conversation_cursor, account_id,
             provider_chat_id, provider_message_id, updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (source_cursor) DO UPDATE SET
            updated_at_unix_seconds = GREATEST(
                makosh_data.telegram_delivery_route_messages.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.telegram_delivery_route_messages.account_cursor =
                   EXCLUDED.account_cursor
           AND makosh_data.telegram_delivery_route_messages.conversation_cursor =
                   EXCLUDED.conversation_cursor
           AND makosh_data.telegram_delivery_route_messages.account_id = EXCLUDED.account_id
           AND makosh_data.telegram_delivery_route_messages.provider_chat_id =
                   EXCLUDED.provider_chat_id
           AND makosh_data.telegram_delivery_route_messages.provider_message_id =
                   EXCLUDED.provider_message_id",
    )
    .bind(locator.source_cursor.as_slice())
    .bind(locator.account_cursor.as_slice())
    .bind(locator.conversation_cursor.as_slice())
    .bind(&locator.account_id)
    .bind(&locator.provider_chat_id)
    .bind(&locator.provider_message_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramDurablePersistenceError::Database)?;
    if message.rows_affected() != 1 {
        return Err(TelegramDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }
    Ok(())
}

fn valid_locator(value: &TelegramDeliveryRouteLocatorV1) -> bool {
    value.account_cursor.iter().any(|byte| *byte != 0)
        && value.conversation_cursor.iter().any(|byte| *byte != 0)
        && value.source_cursor.iter().any(|byte| *byte != 0)
        && valid_text(&value.account_id, 256)
        && valid_text(&value.provider_chat_id, 512)
        && valid_text(&value.provider_message_id, 512)
}

fn valid_text(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use makosh_telegram_api::{TelegramDeliveryState, TelegramMessageReferences};

    use super::*;

    #[test]
    fn locator_reproduces_the_exact_communications_scope_cursors() {
        let message = TelegramMessageProjection {
            sender_source_identity: None,
            message_id: "local-message-1".to_owned(),
            account_id: "account-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: "message-1".to_owned(),
            provider_topic_id: None,
            sender_id: "sender-1".to_owned(),
            sender_display_name: None,
            text: Some("hello".to_owned()),
            media: None,
            references: TelegramMessageReferences::default(),
            observed_at_unix_seconds: 1_700_000_000,
            delivery_state: TelegramDeliveryState::Received,
        };
        let locator = TelegramDeliveryRouteLocatorV1::from_message(&message).expect("locator");
        assert_eq!(
            locator.account_cursor,
            account_source_cursor_v1(ProviderProvenanceV1::Telegram, "account-1")
                .expect("account cursor")
        );
        assert_eq!(
            locator.conversation_cursor,
            conversation_source_cursor_v1(ProviderProvenanceV1::Telegram, "account-1", "chat-1")
                .expect("conversation cursor")
        );
        assert!(valid_locator(&locator));
    }

    #[test]
    fn schema_is_telegram_owned_and_has_no_cross_owner_foreign_keys() {
        assert!(TELEGRAM_SCHEMA_V2.contains("telegram_delivery_route_accounts"));
        assert!(TELEGRAM_SCHEMA_V2.contains("telegram_delivery_route_conversations"));
        assert!(TELEGRAM_SCHEMA_V2.contains("telegram_delivery_route_messages"));
        assert!(!TELEGRAM_SCHEMA_V2.contains("communications_"));
        assert!(!TELEGRAM_SCHEMA_V2.contains("REFERENCES"));
    }
}
