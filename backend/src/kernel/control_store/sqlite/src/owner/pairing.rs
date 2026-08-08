//! SQLite persistence for the one-shot server bootstrap pairing ceremony.

use makosh_kernel_control_store::{InitialOwnerIdentity, ServerBootstrapPairing};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn begin_server_bootstrap_pairing(
        &self,
        pairing: &ServerBootstrapPairing,
        now_unix_ms: u64,
    ) -> Result<(), StoreError> {
        let now = unix_ms(now_unix_ms)?;
        let expires_at = unix_ms(pairing.expires_at_unix_ms())?;
        if expires_at <= now || pairing.challenge().iter().all(|byte| *byte == 0) {
            return Err(StoreError::InvalidServerBootstrapPairing);
        }
        let pairing = pairing.clone();
        self.with_connection(move |connection| begin_pairing(connection, &pairing, now, expires_at))
    }

    pub fn claim_initial_owner_from_server_bootstrap_pairing(
        &self,
        identity: &InitialOwnerIdentity,
        presented_token_sha256: &[u8; 32],
        now_unix_ms: u64,
    ) -> Result<(), StoreError> {
        if !valid_initial_owner_identity(identity) {
            return Err(StoreError::InvalidInitialOwnerIdentity);
        }
        let now = unix_ms(now_unix_ms)?;
        let identity = identity.clone();
        let presented_token_sha256 = *presented_token_sha256;
        self.with_connection(move |connection| {
            claim_pairing(connection, &identity, &presented_token_sha256, now)
        })
    }
}

fn begin_pairing(
    connection: &mut rusqlite::Connection,
    pairing: &ServerBootstrapPairing,
    now: i64,
    expires_at: i64,
) -> Result<(), StoreError> {
    let transaction = connection.transaction()?;
    if initial_owner_exists(&transaction)? {
        return Err(StoreError::InitialOwnerAlreadyClaimed);
    }
    let existing = transaction
            .query_row(
                "SELECT status, expires_at_unix_ms FROM makosh_kernel_server_bootstrap_pairing WHERE singleton = 1",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;
    if matches!(existing, Some((ref status, expires_at)) if status == "active" && expires_at > now)
    {
        return Err(StoreError::ServerBootstrapPairingAlreadyActive);
    }
    transaction.execute(
            "INSERT INTO makosh_kernel_server_bootstrap_pairing (singleton, token_sha256, certificate_sha256, challenge, expires_at_unix_ms, status) VALUES (1, ?1, ?2, ?3, ?4, 'active') ON CONFLICT(singleton) DO UPDATE SET token_sha256 = excluded.token_sha256, certificate_sha256 = excluded.certificate_sha256, challenge = excluded.challenge, expires_at_unix_ms = excluded.expires_at_unix_ms, status = 'active'",
            params![
                pairing.token_sha256().as_slice(),
                pairing.certificate_sha256().as_slice(),
                pairing.challenge().as_slice(),
                expires_at,
            ],
        )?;
    transaction.commit()?;
    Ok(())
}

fn claim_pairing(
    connection: &mut rusqlite::Connection,
    identity: &InitialOwnerIdentity,
    presented_token_sha256: &[u8; 32],
    now: i64,
) -> Result<(), StoreError> {
    let transaction = connection.transaction()?;
    if initial_owner_exists(&transaction)? {
        return Err(StoreError::InitialOwnerAlreadyClaimed);
    }
    let pairing = transaction
            .query_row(
                "SELECT token_sha256, expires_at_unix_ms, status FROM makosh_kernel_server_bootstrap_pairing WHERE singleton = 1",
                [],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
            )
            .optional()?
            .ok_or(StoreError::ServerBootstrapPairingMissing)?;
    let token_sha256: [u8; 32] = pairing
        .0
        .try_into()
        .map_err(|_| StoreError::InvalidServerBootstrapPairing)?;
    if pairing.2 != "active" {
        return Err(StoreError::InitialOwnerAlreadyClaimed);
    }
    if pairing.1 <= now {
        transaction.execute(
                "UPDATE makosh_kernel_server_bootstrap_pairing SET status = 'expired' WHERE singleton = 1 AND status = 'active'",
                [],
            )?;
        transaction.commit()?;
        return Err(StoreError::ServerBootstrapPairingExpired);
    }
    if !constant_time_equal(&token_sha256, presented_token_sha256) {
        return Err(StoreError::ServerBootstrapPairingTokenRejected);
    }
    let claimed = transaction.execute(
            "INSERT OR IGNORE INTO makosh_kernel_initial_owner_identity (singleton, owner_id, device_id, public_key_sec1) VALUES (1, ?1, ?2, ?3)",
            params![identity.owner_id(), identity.device_id(), identity.public_key_sec1().as_slice()],
        )?;
    if claimed != 1 {
        return Err(StoreError::InitialOwnerAlreadyClaimed);
    }
    let consumed = transaction.execute(
            "UPDATE makosh_kernel_server_bootstrap_pairing SET status = 'consumed' WHERE singleton = 1 AND status = 'active'",
            [],
        )?;
    if consumed != 1 {
        return Err(StoreError::ServerBootstrapPairingMissing);
    }
    transaction.commit()?;
    Ok(())
}

fn initial_owner_exists(transaction: &rusqlite::Transaction<'_>) -> Result<bool, StoreError> {
    transaction
        .query_row(
            "SELECT 1 FROM makosh_kernel_initial_owner_identity WHERE singleton = 1",
            [],
            |_| Ok(()),
        )
        .optional()
        .map(|value| value.is_some())
        .map_err(StoreError::from)
}

fn valid_initial_owner_identity(identity: &InitialOwnerIdentity) -> bool {
    valid_identity_token(identity.owner_id())
        && valid_identity_token(identity.device_id())
        && identity.public_key_sec1()[0] == 0x04
}

fn unix_ms(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidServerBootstrapPairing)
}

fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}
