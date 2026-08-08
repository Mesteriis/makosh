use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::model::RecoveryCatalogCandidateV1;

const VAULT_VERSION: u16 = 1;
const MASTER_KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 24;
const MAX_SECRET_BYTES: usize = 65_536;
pub(crate) fn decrypt_candidate_secret(
    vault_path: &Path,
    candidate: &RecoveryCatalogCandidateV1,
    master_key: &[u8; MASTER_KEY_BYTES],
) -> LegacyProviderRecoveryResultV1<Zeroizing<Vec<u8>>> {
    decrypt_candidate_secret_with_key(vault_path, candidate, master_key)
}

pub(crate) fn decode_master_key_file(
    bytes: &[u8],
) -> LegacyProviderRecoveryResultV1<Zeroizing<[u8; MASTER_KEY_BYTES]>> {
    let encoded =
        std::str::from_utf8(bytes).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSecret)?;
    let mut decoded = Zeroizing::new(
        BASE64_STANDARD
            .decode(encoded.trim())
            .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSecret)?,
    );
    if decoded.len() != MASTER_KEY_BYTES {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    let mut key = Zeroizing::new([0_u8; MASTER_KEY_BYTES]);
    key.copy_from_slice(&decoded);
    decoded.fill(0);
    Ok(key)
}

fn decrypt_candidate_secret_with_key(
    vault_path: &Path,
    candidate: &RecoveryCatalogCandidateV1,
    master_key: &[u8; MASTER_KEY_BYTES],
) -> LegacyProviderRecoveryResultV1<Zeroizing<Vec<u8>>> {
    let connection = Connection::open_with_flags(
        vault_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?;
    let row = connection
        .query_row(
            r#"
            SELECT e.version, e.entry_kind, e.account_id, e.purpose,
                   e.nonce, e.ciphertext, e.aad,
                   m.secret_kind, m.store_kind
            FROM vault_entries e
            JOIN account_secret_manifest m ON m.secret_ref = e.secret_ref
            WHERE e.secret_ref = ?1
            "#,
            [&candidate.legacy_secret.secret_ref],
            |row| {
                Ok(StoredSecretV1 {
                    version: row.get(0)?,
                    entry_kind: row.get(1)?,
                    account_id: row.get(2)?,
                    purpose: row.get(3)?,
                    nonce: row.get(4)?,
                    ciphertext: row.get(5)?,
                    aad: row.get(6)?,
                    secret_kind: row.get(7)?,
                    store_kind: row.get(8)?,
                })
            },
        )
        .optional()
        .map_err(|_| LegacyProviderRecoveryErrorV1::DatabaseUnavailable)?
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidSecret)?;
    validate_row(&row, candidate)?;

    let domain_key = derive_domain_key(master_key, b"encryption")?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(domain_key.as_ref()));
    let nonce = BASE64_STANDARD
        .decode(&row.nonce)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSecret)?;
    let ciphertext = BASE64_STANDARD
        .decode(&row.ciphertext)
        .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidSecret)?;
    if nonce.len() != NONCE_BYTES || ciphertext.is_empty() || ciphertext.len() > MAX_SECRET_BYTES {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: row.aad.as_bytes(),
            },
        )
        .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
    if plaintext.is_empty()
        || plaintext.len() > MAX_SECRET_BYTES
        || std::str::from_utf8(&plaintext).is_err()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    Ok(Zeroizing::new(plaintext))
}

fn validate_row(
    row: &StoredSecretV1,
    candidate: &RecoveryCatalogCandidateV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let expected_aad = format!(
        "v={VAULT_VERSION};ref={};kind={};account_id={};purpose={};secret_kind={}",
        candidate.legacy_secret.secret_ref,
        row.entry_kind,
        candidate.account_id,
        candidate.legacy_secret.purpose,
        candidate.legacy_secret.secret_kind,
    );
    if row.version != VAULT_VERSION
        || row.account_id != candidate.account_id
        || row.purpose != candidate.legacy_secret.purpose
        || row.secret_kind != candidate.legacy_secret.secret_kind
        || row.store_kind != "host_vault"
        || row.store_kind != candidate.legacy_secret.store_kind
        || row.aad != expected_aad
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    Ok(())
}

fn derive_domain_key(
    master_key: &[u8; MASTER_KEY_BYTES],
    label: &[u8],
) -> LegacyProviderRecoveryResultV1<Zeroizing<[u8; MASTER_KEY_BYTES]>> {
    let hkdf = Hkdf::<Sha256>::new(None, master_key);
    let mut key = Zeroizing::new([0_u8; MASTER_KEY_BYTES]);
    let mut info = b"makosh-host-vault:v1:".to_vec();
    info.extend_from_slice(label);
    hkdf.expand(&info, key.as_mut())
        .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
    Ok(key)
}

struct StoredSecretV1 {
    version: u16,
    entry_kind: String,
    account_id: String,
    purpose: String,
    nonce: String,
    ciphertext: String,
    aad: String,
    secret_kind: String,
    store_kind: String,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chacha20poly1305::aead::{Aead, AeadCore, OsRng};
    use rusqlite::params;
    use tempfile::tempdir;

    use super::*;
    use crate::model::{
        LegacyProviderCandidateKindV1, RecoveryCatalogConfigurationV1,
        RecoveryCatalogSecretBindingV1,
    };

    #[test]
    fn decrypts_only_the_exact_bound_legacy_secret() {
        let temporary = tempdir().expect("create temporary directory");
        let vault = temporary.path().join("vault.db");
        let connection = Connection::open(&vault).expect("open test Vault");
        connection
            .execute_batch(
                r#"
                CREATE TABLE vault_entries (
                    secret_ref TEXT PRIMARY KEY,
                    entry_kind TEXT NOT NULL,
                    account_id TEXT NOT NULL,
                    purpose TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    nonce TEXT NOT NULL,
                    ciphertext TEXT NOT NULL,
                    aad TEXT NOT NULL
                );
                CREATE TABLE account_secret_manifest (
                    secret_ref TEXT PRIMARY KEY,
                    secret_kind TEXT NOT NULL,
                    store_kind TEXT NOT NULL
                );
                "#,
            )
            .expect("create test Vault schema");
        let candidate = candidate();
        let master_key = [7_u8; MASTER_KEY_BYTES];
        let domain_key =
            derive_domain_key(&master_key, b"encryption").expect("derive test domain key");
        let cipher = XChaCha20Poly1305::new(Key::from_slice(domain_key.as_ref()));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = format!(
            "v=1;ref={};kind=provider_account;account_id={};purpose=imap_password;secret_kind=password",
            candidate.legacy_secret.secret_ref, candidate.account_id
        );
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: b"private-app-password",
                    aad: aad.as_bytes(),
                },
            )
            .expect("encrypt test secret");
        connection
            .execute(
                "INSERT INTO vault_entries VALUES (?1, 'provider_account', ?2, 'imap_password', 1, ?3, ?4, ?5)",
                params![
                    candidate.legacy_secret.secret_ref,
                    candidate.account_id,
                    BASE64_STANDARD.encode(nonce),
                    BASE64_STANDARD.encode(ciphertext),
                    aad,
                ],
            )
            .expect("insert encrypted test secret");
        connection
            .execute(
                "INSERT INTO account_secret_manifest VALUES (?1, 'password', 'host_vault')",
                [&candidate.legacy_secret.secret_ref],
            )
            .expect("insert test manifest");
        drop(connection);

        let secret = decrypt_candidate_secret_with_key(&vault, &candidate, &master_key)
            .expect("decrypt exact bound secret");
        assert_eq!(secret.as_slice(), b"private-app-password");
        assert_eq!(
            decrypt_candidate_secret_with_key(&vault, &candidate, &[8_u8; MASTER_KEY_BYTES]),
            Err(LegacyProviderRecoveryErrorV1::CryptographyUnavailable)
        );

        fs::remove_file(vault).expect("remove test Vault");
    }

    fn candidate() -> RecoveryCatalogCandidateV1 {
        RecoveryCatalogCandidateV1 {
            kind: LegacyProviderCandidateKindV1::Icloud,
            source_account_digest_sha256: "a".repeat(64),
            account_id: "icloud-source".to_owned(),
            display_name: "iCloud".to_owned(),
            external_account_id: "owner@example.test".to_owned(),
            configuration: RecoveryCatalogConfigurationV1::Icloud {
                imap_host: "imap.mail.me.com".to_owned(),
                imap_port: 993,
                tls: true,
                username: "owner@example.test".to_owned(),
            },
            legacy_secret: RecoveryCatalogSecretBindingV1 {
                purpose: "imap_password".to_owned(),
                secret_ref: "secret:icloud-source:imap".to_owned(),
                secret_kind: "password".to_owned(),
                store_kind: "host_vault".to_owned(),
            },
        }
    }
}
