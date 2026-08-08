use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};

use super::HostVault;
use super::constants::VAULT_VERSION;
use super::errors::HostVaultError;
use super::files::ensure_secure_file;
use super::models::StoredVaultEntry;

impl HostVault {
    pub(super) fn initialize_database(&self) -> Result<(), HostVaultError> {
        let connection = self.connection()?;
        connection.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS vault_entries (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS account_secret_manifest (
                secret_ref TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                account_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                secret_kind TEXT NOT NULL,
                store_kind TEXT NOT NULL,
                label TEXT NOT NULL,
                metadata TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vault_check (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                nonce TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                aad TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        ensure_secure_file(&self.database_path())?;
        Ok(())
    }

    pub(super) fn connection(&self) -> Result<Connection, HostVaultError> {
        Ok(Connection::open(self.database_path())?)
    }

    pub(super) fn database_path(&self) -> PathBuf {
        self.home.join("vault.db")
    }

    pub(super) fn recovery_file_path(&self) -> PathBuf {
        self.home.join("makosh-recovery.key")
    }

    pub(super) fn write_vault_check(&self) -> Result<(), HostVaultError> {
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = format!("vault_check:v{VAULT_VERSION}");
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: b"makosh-host-vault",
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        self.connection()?.execute(
            r#"
            INSERT INTO vault_check (id, version, nonce, ciphertext, aad, updated_at)
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id)
            DO UPDATE SET
                version = excluded.version,
                nonce = excluded.nonce,
                ciphertext = excluded.ciphertext,
                aad = excluded.aad,
                updated_at = excluded.updated_at
            "#,
            params![
                VAULT_VERSION,
                BASE64_STANDARD.encode(nonce.as_slice()),
                BASE64_STANDARD.encode(ciphertext),
                aad,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub(super) fn read_vault_check(&self) -> Result<(), HostVaultError> {
        let Some(row) = self
            .connection()?
            .query_row(
                "SELECT version, nonce, ciphertext, aad FROM vault_check WHERE id = 1",
                [],
                |row| {
                    Ok(StoredVaultEntry {
                        version: row.get(0)?,
                        nonce: row.get(1)?,
                        ciphertext: row.get(2)?,
                        aad: row.get(3)?,
                    })
                },
            )
            .optional()?
        else {
            return Ok(());
        };
        if row.version != VAULT_VERSION {
            return Err(HostVaultError::UnsupportedVaultVersion(row.version));
        }
        let key = self.domain_key(b"integrity")?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let nonce = BASE64_STANDARD
            .decode(row.nonce)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        let ciphertext = BASE64_STANDARD
            .decode(row.ciphertext)
            .map_err(|_| HostVaultError::InvalidEncoding)?;
        cipher
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &ciphertext,
                    aad: row.aad.as_bytes(),
                },
            )
            .map_err(|_| HostVaultError::Crypto)?;
        Ok(())
    }
}
