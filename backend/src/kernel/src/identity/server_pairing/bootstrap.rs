//! Ceremony orchestration for a one-shot Linux server bootstrap pairing listener.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_kernel_control_store::ServerBootstrapPairing;
use sha2::{Digest, Sha256};

use crate::identity::enrollment::store::prepare_pristine;
use crate::identity::server_pairing::listener::{
    self as server_pairing_listener, ListenerConfig, TlsServer,
};

const MAX_TTL_SECONDS: u64 = 900;

pub fn run(
    data_dir_override: Option<PathBuf>,
    listen_address: SocketAddr,
    ttl_seconds: u64,
) -> Result<(), String> {
    if ttl_seconds == 0 || ttl_seconds > MAX_TTL_SECONDS {
        return Err(format!(
            "pairing TTL must be between 1 and {MAX_TTL_SECONDS} seconds"
        ));
    }
    let (_data_dir, _lock, store) = prepare_pristine(data_dir_override)?;
    let tls_server = TlsServer::generate()?;
    let mut token = [0_u8; 32];
    let mut challenge = [0_u8; 32];
    getrandom::fill(&mut token).map_err(|error| error.to_string())?;
    getrandom::fill(&mut challenge).map_err(|error| error.to_string())?;
    let now = unix_time_ms()?;
    let ttl_ms = ttl_seconds
        .checked_mul(1_000)
        .ok_or_else(|| "pairing TTL overflow".to_owned())?;
    let expires_at = now
        .checked_add(ttl_ms)
        .ok_or_else(|| "pairing expiry overflow".to_owned())?;
    let token_sha256: [u8; 32] = Sha256::digest(token).into();
    let pairing = ServerBootstrapPairing::new(
        token_sha256,
        *tls_server.certificate_sha256(),
        challenge,
        expires_at,
    );
    store
        .begin_server_bootstrap_pairing(&pairing, now)
        .map_err(|error| format!("{error:?}"))?;
    server_pairing_listener::listen(ListenerConfig {
        store: &store,
        tls_server,
        listen_address,
        idle_timeout: Duration::from_secs(ttl_seconds),
        token,
        challenge,
    })
}

fn unix_time_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            u64::try_from(duration.as_millis()).map_err(|_| "clock overflow".to_owned())
        })
        .map_err(|_| "system clock is before Unix epoch".to_owned())?
}
