//! Mail-owned delivery-intent route locator projection.
//!
//! The projection reverses only the opaque source cursors that Mail itself
//! emitted. Raw provider locators remain inside the Mail persistence boundary.

use sqlx::{Postgres, Transaction};

use crate::MailDurablePersistenceError;

pub const MAIL_SCHEMA_V18: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_route_accounts (
    account_cursor BYTEA PRIMARY KEY,
    connection_id TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(account_cursor) = 32),
    CHECK (length(connection_id) BETWEEN 1 AND 256)
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_route_conversations (
    conversation_cursor BYTEA PRIMARY KEY,
    account_cursor BYTEA NOT NULL,
    connection_id TEXT NOT NULL,
    provider_thread_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    last_sender TEXT,
    recipients TEXT[] NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(conversation_cursor) = 32),
    CHECK (octet_length(account_cursor) = 32),
    CHECK (length(connection_id) BETWEEN 1 AND 256),
    CHECK (length(provider_thread_id) BETWEEN 1 AND 512),
    CHECK (octet_length(subject) <= 4096),
    CHECK (last_sender IS NULL OR length(last_sender) BETWEEN 1 AND 512),
    CHECK (cardinality(recipients) <= 256)
);

CREATE INDEX IF NOT EXISTS mail_delivery_route_conversations_account_idx
    ON makosh_data.mail_delivery_route_conversations
        (account_cursor, updated_at_unix_seconds DESC);

CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_route_messages (
    source_cursor BYTEA PRIMARY KEY,
    account_cursor BYTEA NOT NULL,
    conversation_cursor BYTEA NOT NULL,
    connection_id TEXT NOT NULL,
    provider_thread_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    sender TEXT,
    recipients TEXT[] NOT NULL,
    subject TEXT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(source_cursor) = 32),
    CHECK (octet_length(account_cursor) = 32),
    CHECK (octet_length(conversation_cursor) = 32),
    CHECK (length(connection_id) BETWEEN 1 AND 256),
    CHECK (length(provider_thread_id) BETWEEN 1 AND 512),
    CHECK (length(provider_message_id) BETWEEN 1 AND 512),
    CHECK (sender IS NULL OR length(sender) BETWEEN 1 AND 512),
    CHECK (cardinality(recipients) <= 256),
    CHECK (octet_length(subject) <= 4096)
);

CREATE INDEX IF NOT EXISTS mail_delivery_route_messages_conversation_idx
    ON makosh_data.mail_delivery_route_messages
        (conversation_cursor, updated_at_unix_seconds DESC);
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryRouteLocatorV1 {
    pub account_cursor: [u8; 32],
    pub conversation_cursor: [u8; 32],
    pub source_cursor: [u8; 32],
    pub connection_id: String,
    pub provider_thread_id: String,
    pub provider_message_id: String,
    pub sender: Option<String>,
    pub recipients: Vec<String>,
    pub subject: String,
}

pub(crate) async fn upsert_delivery_route_locator(
    transaction: &mut Transaction<'_, Postgres>,
    locator: &MailDeliveryRouteLocatorV1,
    updated_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    if !valid_locator(locator) || updated_at_unix_seconds <= 0 {
        return Err(MailDurablePersistenceError::InvalidRow);
    }

    let account = sqlx::query(
        "INSERT INTO makosh_data.mail_delivery_route_accounts
            (account_cursor, connection_id, active, updated_at_unix_seconds)
         VALUES ($1, $2, TRUE, $3)
         ON CONFLICT (account_cursor) DO UPDATE SET
            active = TRUE,
            updated_at_unix_seconds = GREATEST(
                makosh_data.mail_delivery_route_accounts.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.mail_delivery_route_accounts.connection_id = EXCLUDED.connection_id",
    )
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.connection_id)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    if account.rows_affected() != 1 {
        return Err(MailDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }

    let conversation = sqlx::query(
        "INSERT INTO makosh_data.mail_delivery_route_conversations
            (conversation_cursor, account_cursor, connection_id, provider_thread_id,
             subject, last_sender, recipients, updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (conversation_cursor) DO UPDATE SET
            subject = EXCLUDED.subject,
            last_sender = EXCLUDED.last_sender,
            recipients = EXCLUDED.recipients,
            updated_at_unix_seconds = GREATEST(
                makosh_data.mail_delivery_route_conversations.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.mail_delivery_route_conversations.account_cursor = EXCLUDED.account_cursor
           AND makosh_data.mail_delivery_route_conversations.connection_id = EXCLUDED.connection_id
           AND makosh_data.mail_delivery_route_conversations.provider_thread_id =
               EXCLUDED.provider_thread_id",
    )
    .bind(locator.conversation_cursor.as_slice())
    .bind(locator.account_cursor.as_slice())
    .bind(&locator.connection_id)
    .bind(&locator.provider_thread_id)
    .bind(&locator.subject)
    .bind(locator.sender.as_deref())
    .bind(&locator.recipients)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    if conversation.rows_affected() != 1 {
        return Err(MailDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }

    let message = sqlx::query(
        "INSERT INTO makosh_data.mail_delivery_route_messages
            (source_cursor, account_cursor, conversation_cursor, connection_id,
             provider_thread_id, provider_message_id, sender, recipients, subject,
             updated_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (source_cursor) DO UPDATE SET
            sender = EXCLUDED.sender,
            recipients = EXCLUDED.recipients,
            subject = EXCLUDED.subject,
            updated_at_unix_seconds = GREATEST(
                makosh_data.mail_delivery_route_messages.updated_at_unix_seconds,
                EXCLUDED.updated_at_unix_seconds
            )
         WHERE makosh_data.mail_delivery_route_messages.account_cursor = EXCLUDED.account_cursor
           AND makosh_data.mail_delivery_route_messages.conversation_cursor =
               EXCLUDED.conversation_cursor
           AND makosh_data.mail_delivery_route_messages.connection_id = EXCLUDED.connection_id
           AND makosh_data.mail_delivery_route_messages.provider_thread_id =
               EXCLUDED.provider_thread_id
           AND makosh_data.mail_delivery_route_messages.provider_message_id =
               EXCLUDED.provider_message_id",
    )
    .bind(locator.source_cursor.as_slice())
    .bind(locator.account_cursor.as_slice())
    .bind(locator.conversation_cursor.as_slice())
    .bind(&locator.connection_id)
    .bind(&locator.provider_thread_id)
    .bind(&locator.provider_message_id)
    .bind(locator.sender.as_deref())
    .bind(&locator.recipients)
    .bind(&locator.subject)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    if message.rows_affected() != 1 {
        return Err(MailDurablePersistenceError::ConflictingDeliveryRouteLocator);
    }
    Ok(())
}

fn valid_locator(value: &MailDeliveryRouteLocatorV1) -> bool {
    value.account_cursor.iter().any(|byte| *byte != 0)
        && value.conversation_cursor.iter().any(|byte| *byte != 0)
        && value.source_cursor.iter().any(|byte| *byte != 0)
        && valid_text(&value.connection_id, 256)
        && valid_text(&value.provider_thread_id, 512)
        && valid_text(&value.provider_message_id, 512)
        && value
            .sender
            .as_deref()
            .is_none_or(|sender| valid_text(sender, 512))
        && value.recipients.len() <= 256
        && value
            .recipients
            .iter()
            .all(|recipient| valid_text(recipient, 512))
        && value.subject.len() <= 4096
}

fn valid_text(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_requires_three_distinct_non_empty_provider_identities() {
        let mut locator = MailDeliveryRouteLocatorV1 {
            account_cursor: [1; 32],
            conversation_cursor: [2; 32],
            source_cursor: [3; 32],
            connection_id: "connection-1".to_owned(),
            provider_thread_id: "thread-1".to_owned(),
            provider_message_id: "message-1".to_owned(),
            sender: Some("sender@example.com".to_owned()),
            recipients: vec!["owner@example.com".to_owned()],
            subject: "Subject".to_owned(),
        };
        assert!(valid_locator(&locator));
        locator.source_cursor = [0; 32];
        assert!(!valid_locator(&locator));
    }

    #[test]
    fn schema_is_mail_owned_and_contains_no_cross_owner_foreign_keys() {
        assert!(MAIL_SCHEMA_V18.contains("makosh_data.mail_delivery_route_accounts"));
        assert!(MAIL_SCHEMA_V18.contains("makosh_data.mail_delivery_route_conversations"));
        assert!(MAIL_SCHEMA_V18.contains("makosh_data.mail_delivery_route_messages"));
        assert!(!MAIL_SCHEMA_V18.contains("communications_"));
        assert!(!MAIL_SCHEMA_V18.contains("FOREIGN KEY"));
    }
}
