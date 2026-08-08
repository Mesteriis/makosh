//! Mail-owned provider locator state.
//!
//! Public clients address a stable `message_id`. Provider-specific IMAP
//! mailbox/UIDVALIDITY/UID coordinates stay private to this persistence and
//! the IMAP adapter.

use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};

use crate::{MailDurablePersistence, MailDurablePersistenceError};

pub const MAIL_SCHEMA_V13: &str = r#"
ALTER TABLE makosh_data.mail_operational_messages
    ADD COLUMN message_id TEXT
    GENERATED ALWAYS AS (provider_message_id) STORED;
ALTER TABLE makosh_data.mail_operational_message_folders
    ADD COLUMN message_id TEXT
    GENERATED ALWAYS AS (provider_message_id) STORED;
ALTER TABLE makosh_data.mail_message_flag_operations
    ADD COLUMN message_id TEXT
    GENERATED ALWAYS AS (provider_message_id) STORED;

CREATE UNIQUE INDEX IF NOT EXISTS mail_operational_messages_stable_message_idx
    ON makosh_data.mail_operational_messages (connection_id, message_id);

CREATE TABLE IF NOT EXISTS makosh_data.mail_imap_message_locators (
    connection_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    mailbox_id TEXT NOT NULL,
    uid_validity BIGINT NOT NULL CHECK (uid_validity > 0 AND uid_validity <= 4294967295),
    uid BIGINT NOT NULL CHECK (uid > 0 AND uid <= 4294967295),
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    PRIMARY KEY (connection_id, message_id),
    UNIQUE (connection_id, mailbox_id, uid_validity, uid),
    FOREIGN KEY (connection_id, message_id)
        REFERENCES makosh_data.mail_operational_messages (connection_id, message_id)
        ON DELETE CASCADE,
    CHECK (connection_id <> ''),
    CHECK (message_id <> ''),
    CHECK (mailbox_id <> ''),
    CHECK (octet_length(mailbox_id) <= 512),
    CHECK (mailbox_id !~ '[\x00\r\n]')
);
"#;

pub const MAIL_SCHEMA_V14: &str = r#"
CREATE INDEX IF NOT EXISTS mail_operational_message_folders_message_idx
    ON makosh_data.mail_operational_message_folders (connection_id, message_id, folder_id);
CREATE INDEX IF NOT EXISTS mail_operational_messages_thread_stable_idx
    ON makosh_data.mail_operational_messages
    (connection_id, provider_thread_id, updated_at_unix_seconds DESC, message_id DESC);
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailImapMessageLocatorV1 {
    pub mailbox_id: String,
    pub uid_validity: u32,
    pub uid: u32,
}

pub fn initial_imap_message_id(
    connection_id: &str,
    locator: &MailImapMessageLocatorV1,
) -> Result<String, MailDurablePersistenceError> {
    validate_imap_locator(locator)?;
    if !valid_id(connection_id) {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    let mut digest = Sha256::new();
    digest.update(b"makosh-mail-imap-message-v1\0");
    digest.update(connection_id.as_bytes());
    digest.update(b"\0");
    digest.update(locator.mailbox_id.as_bytes());
    digest.update(b"\0");
    digest.update(locator.uid_validity.to_be_bytes());
    digest.update(locator.uid.to_be_bytes());
    Ok(format!("imap:v1:{}", hex_digest(&digest.finalize())))
}

impl MailDurablePersistence {
    pub async fn recent_inbox_imap_uids(
        &self,
        connection_id: &str,
        limit: u32,
    ) -> Result<Vec<u32>, MailDurablePersistenceError> {
        if !valid_id(connection_id) || !(1..=1_000).contains(&limit) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let rows = sqlx::query(
            "SELECT locator.uid \
             FROM makosh_data.mail_imap_message_locators AS locator \
             JOIN makosh_data.mail_operational_messages AS message \
               ON message.connection_id = locator.connection_id \
              AND message.message_id = locator.message_id \
             WHERE locator.connection_id = $1 \
               AND lower(locator.mailbox_id) = 'inbox' \
             ORDER BY message.sent_at_unix_seconds DESC NULLS LAST, \
                      message.cursor_sequence DESC \
             LIMIT $2",
        )
        .bind(connection_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        rows.iter()
            .map(|row| {
                let uid = row
                    .try_get::<i64, _>("uid")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                u32::try_from(uid).map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn imap_message_locator(
        &self,
        connection_id: &str,
        message_id: &str,
    ) -> Result<Option<MailImapMessageLocatorV1>, MailDurablePersistenceError> {
        if !valid_id(connection_id) || !valid_id(message_id) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "SELECT mailbox_id, uid_validity, uid \
             FROM makosh_data.mail_imap_message_locators \
             WHERE connection_id = $1 AND message_id = $2",
        )
        .bind(connection_id)
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        row.as_ref().map(locator_from_row).transpose()
    }

    pub async fn resolve_imap_message_id(
        &self,
        connection_id: &str,
        locator: &MailImapMessageLocatorV1,
        legacy_message_id: &str,
    ) -> Result<Option<String>, MailDurablePersistenceError> {
        validate_imap_locator(locator)?;
        if !valid_id(connection_id) || !valid_id(legacy_message_id) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let exact = sqlx::query(
            "SELECT message_id FROM makosh_data.mail_imap_message_locators \
             WHERE connection_id = $1 AND mailbox_id = $2 \
               AND uid_validity = $3 AND uid = $4",
        )
        .bind(connection_id)
        .bind(&locator.mailbox_id)
        .bind(i64::from(locator.uid_validity))
        .bind(i64::from(locator.uid))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = exact {
            return row
                .try_get::<String, _>("message_id")
                .map(Some)
                .map_err(|_| MailDurablePersistenceError::InvalidRow);
        }

        let legacy = sqlx::query(
            "SELECT message.message_id \
             FROM makosh_data.mail_operational_messages AS message \
             WHERE message.connection_id = $1 AND message.message_id = $2 \
               AND NOT EXISTS ( \
                 SELECT 1 FROM makosh_data.mail_imap_message_locators AS locator \
                 WHERE locator.connection_id = message.connection_id \
                   AND locator.message_id = message.message_id \
               )",
        )
        .bind(connection_id)
        .bind(legacy_message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        legacy
            .as_ref()
            .map(|row| {
                row.try_get::<String, _>("message_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .transpose()
    }
}

pub(crate) async fn upsert_imap_message_locator(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
    message_id: &str,
    locator: &MailImapMessageLocatorV1,
    observed_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    validate_imap_locator(locator)?;
    if !valid_id(connection_id) || !valid_id(message_id) || observed_at_unix_seconds <= 0 {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    sqlx::query(
        "INSERT INTO makosh_data.mail_imap_message_locators \
         (connection_id, message_id, mailbox_id, uid_validity, uid, observed_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (connection_id, message_id) DO UPDATE SET \
           mailbox_id = EXCLUDED.mailbox_id, \
           uid_validity = EXCLUDED.uid_validity, \
           uid = EXCLUDED.uid, \
           observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds",
    )
    .bind(connection_id)
    .bind(message_id)
    .bind(&locator.mailbox_id)
    .bind(i64::from(locator.uid_validity))
    .bind(i64::from(locator.uid))
    .bind(observed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    Ok(())
}

pub(crate) fn validate_imap_locator(
    locator: &MailImapMessageLocatorV1,
) -> Result<(), MailDurablePersistenceError> {
    if !valid_mailbox_id(&locator.mailbox_id) || locator.uid_validity == 0 || locator.uid == 0 {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    Ok(())
}

fn locator_from_row(row: &PgRow) -> Result<MailImapMessageLocatorV1, MailDurablePersistenceError> {
    let mailbox_id = row
        .try_get::<String, _>("mailbox_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let uid_validity = row
        .try_get::<i64, _>("uid_validity")
        .ok()
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or(MailDurablePersistenceError::InvalidRow)?;
    let uid = row
        .try_get::<i64, _>("uid")
        .ok()
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or(MailDurablePersistenceError::InvalidRow)?;
    let locator = MailImapMessageLocatorV1 {
        mailbox_id,
        uid_validity,
        uid,
    };
    validate_imap_locator(&locator)?;
    Ok(locator)
}

fn valid_id(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 512
        && !value.contains(['\0', '\r', '\n'])
        && value.trim() == value
}

fn valid_mailbox_id(value: &str) -> bool {
    valid_id(value)
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_adds_a_stable_compatibility_alias_and_keeps_imap_locator_private() {
        assert!(
            MAIL_SCHEMA_V13
                .contains("message_id TEXT\n    GENERATED ALWAYS AS (provider_message_id) STORED")
        );
        assert!(!MAIL_SCHEMA_V13.contains("RENAME COLUMN"));
        assert!(MAIL_SCHEMA_V13.contains("mail_imap_message_locators"));
        assert!(MAIL_SCHEMA_V13.contains("UNIQUE (connection_id, mailbox_id, uid_validity, uid)"));
        assert!(MAIL_SCHEMA_V14.contains("mail_operational_message_folders_message_idx"));
        assert!(MAIL_SCHEMA_V14.contains("mail_operational_messages_thread_stable_idx"));
        assert!(!MAIL_SCHEMA_V13.contains("communications"));
        assert!(!MAIL_SCHEMA_V13.contains("credential"));
    }

    #[test]
    fn locator_requires_bounded_exact_mailbox_and_positive_epoch_coordinates() {
        assert!(
            validate_imap_locator(&MailImapMessageLocatorV1 {
                mailbox_id: "Archive/2026".to_owned(),
                uid_validity: 7,
                uid: 42,
            })
            .is_ok()
        );
        assert!(
            validate_imap_locator(&MailImapMessageLocatorV1 {
                mailbox_id: "Archive\r\nEXPUNGE".to_owned(),
                uid_validity: 7,
                uid: 42,
            })
            .is_err()
        );
        assert!(
            validate_imap_locator(&MailImapMessageLocatorV1 {
                mailbox_id: "INBOX".to_owned(),
                uid_validity: 0,
                uid: 42,
            })
            .is_err()
        );
    }

    #[test]
    fn initial_identity_is_stable_and_does_not_expose_private_mailbox_name() {
        let locator = MailImapMessageLocatorV1 {
            mailbox_id: "Private/Archive".to_owned(),
            uid_validity: 7,
            uid: 42,
        };
        let first = initial_imap_message_id("account-1", &locator).expect("initial identity");
        let replay = initial_imap_message_id("account-1", &locator).expect("replayed identity");
        assert_eq!(first, replay);
        assert!(first.starts_with("imap:v1:"));
        assert!(!first.contains("Private"));
        assert_ne!(
            first,
            initial_imap_message_id("account-2", &locator).expect("scoped identity")
        );
    }
}
