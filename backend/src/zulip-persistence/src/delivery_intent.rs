//! Zulip-owned reverse locator for provider-neutral delivery intents.

use makosh_communications_ingress::{
    ProviderProvenanceV1, account_source_cursor_v1, conversation_source_cursor_v1,
    scoped_record_source_cursor_v1,
};
use sqlx::{Postgres, Transaction};

use crate::ZulipDurablePersistenceError;

pub const ZULIP_DELIVERY_ROUTE_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.zulip_delivery_route_accounts (
    account_cursor BYTEA PRIMARY KEY,
    account_id TEXT NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(account_cursor) = 32),
    CHECK (char_length(account_id) BETWEEN 1 AND 256)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_delivery_route_conversations (
    conversation_cursor BYTEA PRIMARY KEY,
    account_cursor BYTEA NOT NULL,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    UNIQUE (account_id, provider_chat_id),
    CHECK (octet_length(conversation_cursor) = 32),
    CHECK (octet_length(account_cursor) = 32),
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 512)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_delivery_route_messages (
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
    CHECK (char_length(account_id) BETWEEN 1 AND 256),
    CHECK (char_length(provider_chat_id) BETWEEN 1 AND 512),
    CHECK (char_length(provider_message_id) BETWEEN 1 AND 512)
);
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipDeliveryRouteLocatorV1 {
    pub account_cursor: [u8; 32],
    pub conversation_cursor: [u8; 32],
    pub source_cursor: [u8; 32],
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
}

impl ZulipDeliveryRouteLocatorV1 {
    pub fn new(
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<Self, ZulipDurablePersistenceError> {
        Ok(Self {
            account_cursor: account_source_cursor_v1(ProviderProvenanceV1::Zulip, account_id)
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            conversation_cursor: conversation_source_cursor_v1(
                ProviderProvenanceV1::Zulip,
                account_id,
                provider_chat_id,
            )
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            source_cursor: scoped_record_source_cursor_v1(
                ProviderProvenanceV1::Zulip,
                account_id,
                provider_message_id,
            )
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            provider_message_id: provider_message_id.to_owned(),
        })
    }
}

pub(crate) async fn upsert_delivery_route_locator(
    transaction: &mut Transaction<'_, Postgres>,
    locator: &ZulipDeliveryRouteLocatorV1,
    updated_at_unix_seconds: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    if !valid_locator(locator) || updated_at_unix_seconds <= 0 {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    let account = sqlx::query(
        "INSERT INTO makosh_data.zulip_delivery_route_accounts
            (account_cursor, account_id, active, updated_at_unix_seconds)
         VALUES ($1, $2, TRUE, $3)
         ON CONFLICT (account_cursor) DO UPDATE SET
            active = TRUE,
            updated_at_unix_seconds = GREATEST(
                makosh_data.zulip_delivery_route_accounts.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds)
         WHERE makosh_data.zulip_delivery_route_accounts.account_id = EXCLUDED.account_id",
    )
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.account_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ZulipDurablePersistenceError::Database)?;
    if account.rows_affected() != 1 {
        return Err(ZulipDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }
    let conversation = sqlx::query(
        "INSERT INTO makosh_data.zulip_delivery_route_conversations
            (conversation_cursor, account_cursor, account_id, provider_chat_id,
             updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (conversation_cursor) DO UPDATE SET
            updated_at_unix_seconds = GREATEST(
                makosh_data.zulip_delivery_route_conversations.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds)
         WHERE makosh_data.zulip_delivery_route_conversations.account_cursor =
                   EXCLUDED.account_cursor
           AND makosh_data.zulip_delivery_route_conversations.account_id =
                   EXCLUDED.account_id
           AND makosh_data.zulip_delivery_route_conversations.provider_chat_id =
                   EXCLUDED.provider_chat_id",
    )
    .bind(locator.conversation_cursor.as_slice())
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.account_id)
    .bind(&locator.provider_chat_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ZulipDurablePersistenceError::Database)?;
    if conversation.rows_affected() != 1 {
        return Err(ZulipDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }
    let message = sqlx::query(
        "INSERT INTO makosh_data.zulip_delivery_route_messages
            (source_cursor, account_cursor, conversation_cursor, account_id,
             provider_chat_id, provider_message_id, updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (source_cursor) DO UPDATE SET
            updated_at_unix_seconds = GREATEST(
                makosh_data.zulip_delivery_route_messages.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds)
         WHERE makosh_data.zulip_delivery_route_messages.account_cursor =
                   EXCLUDED.account_cursor
           AND makosh_data.zulip_delivery_route_messages.conversation_cursor =
                   EXCLUDED.conversation_cursor
           AND makosh_data.zulip_delivery_route_messages.account_id = EXCLUDED.account_id
           AND makosh_data.zulip_delivery_route_messages.provider_chat_id =
                   EXCLUDED.provider_chat_id
           AND makosh_data.zulip_delivery_route_messages.provider_message_id =
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
    .map_err(|_| ZulipDurablePersistenceError::Database)?;
    if message.rows_affected() != 1 {
        return Err(ZulipDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }
    Ok(())
}

fn valid_locator(value: &ZulipDeliveryRouteLocatorV1) -> bool {
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
    use super::*;

    #[test]
    fn locator_reproduces_the_exact_communications_scope_cursors() {
        let locator =
            ZulipDeliveryRouteLocatorV1::new("account-1", "chat-1", "message-1").expect("locator");
        assert_eq!(
            locator.account_cursor,
            account_source_cursor_v1(ProviderProvenanceV1::Zulip, "account-1")
                .expect("account cursor")
        );
        assert_eq!(
            locator.conversation_cursor,
            conversation_source_cursor_v1(ProviderProvenanceV1::Zulip, "account-1", "chat-1",)
                .expect("conversation cursor")
        );
        assert!(valid_locator(&locator));
    }

    #[test]
    fn schema_is_zulip_owned_and_has_no_cross_owner_foreign_keys() {
        assert!(ZULIP_DELIVERY_ROUTE_SCHEMA_V1.contains("zulip_delivery_route_accounts"));
        assert!(!ZULIP_DELIVERY_ROUTE_SCHEMA_V1.contains("communications_"));
        assert!(!ZULIP_DELIVERY_ROUTE_SCHEMA_V1.contains("REFERENCES"));
    }
}
