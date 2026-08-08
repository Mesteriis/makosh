//! Control-plane contract for the optional standalone Zulip connector.
//!
//! This crate deliberately has no database, filesystem, NATS, vault, or HTTP
//! dependencies. Transport and spool implementations belong to the connector
//! composition root and can be tested independently of provider execution.

use makosh_provider_api::{
    CredentialLease, ProviderCommandEnvelope, ProviderCommandResult, ProviderManifest,
    ProviderRuntimePort, ProviderRuntimePortError, RuntimeTopology,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONTROL_PROTOCOL_VERSION: u32 = 1;
pub const COMMAND_SUBJECT_PREFIX: &str = "makosh.provider.commands.v1";
pub const RESULT_SUBJECT_PREFIX: &str = "makosh.provider.results.v1";
pub const OBSERVATION_SUBJECT_PREFIX: &str = "makosh.provider.observations.v1";
pub const ACK_SUBJECT_PREFIX: &str = "makosh.provider.acks.v1";
pub const DEFAULT_SPOOL_MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

pub fn command_subject(provider: &str, account: &str) -> Result<String, SubjectError> {
    subject(&[COMMAND_SUBJECT_PREFIX, provider, account])
}

pub fn result_subject(provider: &str, account: &str) -> Result<String, SubjectError> {
    subject(&[RESULT_SUBJECT_PREFIX, provider, account])
}

pub fn observation_subject(
    provider: &str,
    account: &str,
    kind: &str,
) -> Result<String, SubjectError> {
    subject(&[OBSERVATION_SUBJECT_PREFIX, provider, account, kind])
}

pub fn ack_subject(provider: &str, account: &str) -> Result<String, SubjectError> {
    subject(&[ACK_SUBJECT_PREFIX, provider, account])
}

fn subject(parts: &[&str]) -> Result<String, SubjectError> {
    if parts.iter().skip(1).any(|part| {
        part.trim().is_empty() || part.contains('.') || part.contains('/') || part.contains(' ')
    }) {
        return Err(SubjectError::InvalidSegment);
    }
    Ok(parts.join("."))
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SubjectError {
    #[error("subject segment is empty or contains a reserved delimiter")]
    InvalidSegment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpoolHealth {
    Ready,
    Degraded,
    HardStop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpoolEntry {
    pub stable_id: String,
    pub lease_epoch: u64,
    pub bytes: usize,
    pub acknowledged: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpoolState {
    capacity_bytes: usize,
    degraded_threshold_bytes: usize,
    used_bytes: usize,
    entries: std::collections::BTreeMap<String, SpoolEntry>,
}

impl SpoolState {
    pub fn new(capacity_bytes: usize) -> Result<Self, SpoolError> {
        if capacity_bytes == 0 {
            return Err(SpoolError::InvalidCapacity);
        }
        Ok(Self {
            capacity_bytes,
            degraded_threshold_bytes: capacity_bytes.saturating_mul(80) / 100,
            used_bytes: 0,
            entries: std::collections::BTreeMap::new(),
        })
    }

    pub fn health(&self) -> SpoolHealth {
        if self.used_bytes >= self.capacity_bytes {
            SpoolHealth::HardStop
        } else if self.used_bytes >= self.degraded_threshold_bytes {
            SpoolHealth::Degraded
        } else {
            SpoolHealth::Ready
        }
    }

    pub fn insert(&mut self, entry: SpoolEntry) -> Result<bool, SpoolError> {
        if self.entries.contains_key(&entry.stable_id) {
            return Ok(false);
        }
        if self.used_bytes.saturating_add(entry.bytes) > self.capacity_bytes {
            return Err(SpoolError::CapacityExceeded);
        }
        self.used_bytes += entry.bytes;
        self.entries.insert(entry.stable_id.clone(), entry);
        Ok(true)
    }

    pub fn acknowledge(&mut self, stable_id: &str, lease_epoch: u64) -> Result<(), SpoolError> {
        let entry = self
            .entries
            .get_mut(stable_id)
            .ok_or_else(|| SpoolError::UnknownEntry(stable_id.to_owned()))?;
        if entry.lease_epoch != lease_epoch {
            return Err(SpoolError::StaleLeaseEpoch);
        }
        entry.acknowledged = true;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SpoolError {
    #[error("spool capacity must be greater than zero")]
    InvalidCapacity,
    #[error("spool capacity exceeded")]
    CapacityExceeded,
    #[error("spool entry not found: {0}")]
    UnknownEntry(String),
    #[error("stale lease epoch")]
    StaleLeaseEpoch,
}

/// SQLite-backed durable spool. The connector owns this file; core never sees
/// its path or credentials. Entries remain until the core acknowledgement is
/// recorded with the matching lease epoch.
pub struct SqliteSpool {
    connection: rusqlite::Connection,
    capacity_bytes: usize,
    max_age_secs: u64,
}

impl SqliteSpool {
    pub fn open(
        path: impl AsRef<std::path::Path>,
        capacity_bytes: usize,
    ) -> Result<Self, SpoolDbError> {
        Self::open_with_limits(path, capacity_bytes, DEFAULT_SPOOL_MAX_AGE_SECS)
    }

    pub fn open_with_limits(
        path: impl AsRef<std::path::Path>,
        capacity_bytes: usize,
        max_age_secs: u64,
    ) -> Result<Self, SpoolDbError> {
        if capacity_bytes == 0 {
            return Err(SpoolDbError::InvalidCapacity);
        }
        let connection = rusqlite::Connection::open(path)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS connector_spool (
                stable_id TEXT PRIMARY KEY NOT NULL,
                lease_epoch INTEGER NOT NULL,
                bytes INTEGER NOT NULL,
                acknowledged INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(Self {
            connection,
            capacity_bytes,
            max_age_secs,
        })
    }

    pub fn insert(&self, entry: &SpoolEntry) -> Result<bool, SpoolDbError> {
        let used: usize = self
            .connection
            .query_row(
                "SELECT COALESCE(SUM(bytes), 0) FROM connector_spool",
                [],
                |row| row.get::<_, i64>(0),
            )?
            .try_into()
            .map_err(|_| SpoolDbError::InvalidStoredSize)?;
        if used.saturating_add(entry.bytes) > self.capacity_bytes {
            return Err(SpoolDbError::CapacityExceeded);
        }
        let inserted = self.connection.execute(
            "INSERT OR IGNORE INTO connector_spool (stable_id, lease_epoch, bytes, acknowledged, created_at) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
            rusqlite::params![entry.stable_id, entry.lease_epoch as i64, entry.bytes as i64, entry.acknowledged as i64],
        )?;
        Ok(inserted == 1)
    }

    pub fn acknowledge(&self, stable_id: &str, lease_epoch: u64) -> Result<(), SpoolDbError> {
        let changed = self.connection.execute(
            "UPDATE connector_spool SET acknowledged = 1 WHERE stable_id = ?1 AND lease_epoch = ?2",
            rusqlite::params![stable_id, lease_epoch as i64],
        )?;
        if changed == 1 {
            return Ok(());
        }
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM connector_spool WHERE stable_id = ?1)",
            [stable_id],
            |row| row.get(0),
        )?;
        if exists {
            Err(SpoolDbError::StaleLeaseEpoch)
        } else {
            Err(SpoolDbError::UnknownEntry(stable_id.to_owned()))
        }
    }

    pub fn remove_acknowledged(&self, stable_id: &str) -> Result<bool, SpoolDbError> {
        Ok(self.connection.execute(
            "DELETE FROM connector_spool WHERE stable_id = ?1 AND acknowledged = 1",
            [stable_id],
        )? == 1)
    }

    pub fn purge_expired_acknowledged(&self) -> Result<usize, SpoolDbError> {
        let cutoff = i64::try_from(self.max_age_secs)
            .ok()
            .and_then(|age| {
                i64::try_from(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs(),
                )
                .ok()
                .map(|now| now.saturating_sub(age))
            })
            .ok_or(SpoolDbError::InvalidRetention)?;
        Ok(self.connection.execute(
            "DELETE FROM connector_spool WHERE acknowledged = 1 AND created_at > 0 AND created_at < ?1",
            [cutoff],
        )?)
    }
}

#[derive(Debug, Error)]
pub enum SpoolDbError {
    #[error("invalid spool capacity")]
    InvalidCapacity,
    #[error("spool capacity exceeded")]
    CapacityExceeded,
    #[error("stored spool size is invalid")]
    InvalidStoredSize,
    #[error("spool entry not found: {0}")]
    UnknownEntry(String),
    #[error("stale lease epoch")]
    StaleLeaseEpoch,
    #[error("invalid spool retention")]
    InvalidRetention,
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HelloRequest {
    pub protocol_version: u32,
    pub provider_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HelloResponse {
    pub protocol_version: u32,
    pub manifest: ProviderManifest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ControlRequest {
    Hello(HelloRequest),
    Describe,
    Start,
    Stop,
    Drain,
    Health,
    BeginAuth {
        account_id: String,
    },
    CompleteAuth {
        account_id: String,
        callback_token: String,
    },
    RenewCredentialLease {
        account_id: String,
        epoch: u64,
    },
    RevokeCredentialLease {
        account_id: String,
        epoch: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ControlResponse {
    Hello(HelloResponse),
    Manifest(ProviderManifest),
    Accepted,
    Health { ready: bool, draining: bool },
    Rejected { code: String },
    AuthStarted { account_id: String },
    AuthCompleted { account_id: String },
    LeaseRenewed { account_id: String, epoch: u64 },
    LeaseRevoked { account_id: String, epoch: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorLifecycleState {
    Stopped,
    Running,
    Draining,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectorLifecycle {
    state: ConnectorLifecycleState,
}

impl Default for ConnectorLifecycle {
    fn default() -> Self {
        Self {
            state: ConnectorLifecycleState::Stopped,
        }
    }
}

impl ConnectorLifecycle {
    pub fn state(&self) -> ConnectorLifecycleState {
        self.state
    }

    pub fn handle(
        &mut self,
        request: ControlRequest,
    ) -> Result<ControlResponse, ControlProtocolError> {
        match request {
            ControlRequest::Start if self.state == ConnectorLifecycleState::Stopped => {
                self.state = ConnectorLifecycleState::Running;
                Ok(ControlResponse::Accepted)
            }
            ControlRequest::Drain if self.state == ConnectorLifecycleState::Running => {
                self.state = ConnectorLifecycleState::Draining;
                Ok(ControlResponse::Accepted)
            }
            ControlRequest::Stop if self.state != ConnectorLifecycleState::Stopped => {
                self.state = ConnectorLifecycleState::Stopped;
                Ok(ControlResponse::Accepted)
            }
            ControlRequest::Health => Ok(ControlResponse::Health {
                ready: self.state == ConnectorLifecycleState::Running,
                draining: self.state == ConnectorLifecycleState::Draining,
            }),
            ControlRequest::BeginAuth { account_id } => {
                if account_id.trim().is_empty() {
                    return Err(ControlProtocolError::InvalidAccountId);
                }
                Ok(ControlResponse::AuthStarted { account_id })
            }
            ControlRequest::CompleteAuth {
                account_id,
                callback_token,
            } => {
                if account_id.trim().is_empty() || callback_token.trim().is_empty() {
                    return Err(ControlProtocolError::InvalidAuthRequest);
                }
                Ok(ControlResponse::AuthCompleted { account_id })
            }
            ControlRequest::RenewCredentialLease { account_id, epoch } => {
                if account_id.trim().is_empty() || epoch == 0 {
                    return Err(ControlProtocolError::InvalidLeaseRequest);
                }
                Ok(ControlResponse::LeaseRenewed { account_id, epoch })
            }
            ControlRequest::RevokeCredentialLease { account_id, epoch } => {
                if account_id.trim().is_empty() || epoch == 0 {
                    return Err(ControlProtocolError::InvalidLeaseRequest);
                }
                Ok(ControlResponse::LeaseRevoked { account_id, epoch })
            }
            ControlRequest::Start | ControlRequest::Drain | ControlRequest::Stop => {
                Err(ControlProtocolError::InvalidLifecycleTransition)
            }
            request => handle_control_request(request),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ControlProtocolError {
    #[error("unsupported control protocol version: {0}")]
    UnsupportedVersion(u32),
    #[error("unexpected provider id: {0}")]
    UnexpectedProvider(String),
    #[error("provider manifest does not support shared connector topology")]
    UnsupportedTopology,
    #[error("invalid connector lifecycle transition")]
    InvalidLifecycleTransition,
    #[error("account id is empty")]
    InvalidAccountId,
    #[error("auth request is invalid")]
    InvalidAuthRequest,
    #[error("credential lease request is invalid")]
    InvalidLeaseRequest,
}

pub fn manifest() -> ProviderManifest {
    makosh_provider_zulip::runtime::ZulipInProcessRuntime::new(
        makosh_provider_zulip::runtime::ZulipRuntimeConfig::new(
            "connector-account",
            "http://invalid.local",
            "connector@example.invalid",
        )
        .expect("static connector manifest configuration"),
    )
    .manifest()
}

pub fn handle_control_request(
    request: ControlRequest,
) -> Result<ControlResponse, ControlProtocolError> {
    match request {
        ControlRequest::Hello(hello) => {
            if hello.protocol_version != CONTROL_PROTOCOL_VERSION {
                return Err(ControlProtocolError::UnsupportedVersion(
                    hello.protocol_version,
                ));
            }
            if hello.provider_id != "zulip" {
                return Err(ControlProtocolError::UnexpectedProvider(hello.provider_id));
            }
            Ok(ControlResponse::Hello(HelloResponse {
                protocol_version: CONTROL_PROTOCOL_VERSION,
                manifest: manifest(),
            }))
        }
        ControlRequest::Describe => Ok(ControlResponse::Manifest(manifest())),
        ControlRequest::Start | ControlRequest::Stop | ControlRequest::Drain => {
            Ok(ControlResponse::Accepted)
        }
        ControlRequest::Health => Ok(ControlResponse::Health {
            ready: true,
            draining: false,
        }),
        ControlRequest::BeginAuth { account_id } if !account_id.trim().is_empty() => {
            Ok(ControlResponse::AuthStarted { account_id })
        }
        ControlRequest::CompleteAuth {
            account_id,
            callback_token,
        } if !account_id.trim().is_empty() && !callback_token.trim().is_empty() => {
            Ok(ControlResponse::AuthCompleted { account_id })
        }
        ControlRequest::RenewCredentialLease { account_id, epoch }
            if !account_id.trim().is_empty() && epoch > 0 =>
        {
            Ok(ControlResponse::LeaseRenewed { account_id, epoch })
        }
        ControlRequest::RevokeCredentialLease { account_id, epoch }
            if !account_id.trim().is_empty() && epoch > 0 =>
        {
            Ok(ControlResponse::LeaseRevoked { account_id, epoch })
        }
        ControlRequest::BeginAuth { .. } => Err(ControlProtocolError::InvalidAccountId),
        ControlRequest::CompleteAuth { .. } => Err(ControlProtocolError::InvalidAuthRequest),
        ControlRequest::RenewCredentialLease { .. }
        | ControlRequest::RevokeCredentialLease { .. } => {
            Err(ControlProtocolError::InvalidLeaseRequest)
        }
    }
}

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error(transparent)]
    Protocol(#[from] ControlProtocolError),
    #[error(transparent)]
    Provider(#[from] ProviderRuntimePortError),
}

pub async fn execute_shared_connector_command(
    runtime: &dyn ProviderRuntimePort,
    command: &ProviderCommandEnvelope,
    credential: CredentialLease,
) -> Result<ProviderCommandResult, ConnectorError> {
    let manifest = runtime.manifest();
    if manifest.provider_id != command.provider_id {
        return Err(ConnectorError::Protocol(
            ControlProtocolError::UnexpectedProvider(command.provider_id.as_str().to_owned()),
        ));
    }
    if !manifest.supports(RuntimeTopology::SharedConnector) {
        return Err(ConnectorError::Protocol(
            ControlProtocolError::UnsupportedTopology,
        ));
    }
    runtime
        .execute(command, credential)
        .await
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_fails_closed_on_version_or_provider_mismatch() {
        assert_eq!(
            handle_control_request(ControlRequest::Hello(HelloRequest {
                protocol_version: CONTROL_PROTOCOL_VERSION + 1,
                provider_id: "zulip".to_owned(),
            })),
            Err(ControlProtocolError::UnsupportedVersion(
                CONTROL_PROTOCOL_VERSION + 1,
            ))
        );
        assert_eq!(
            handle_control_request(ControlRequest::Hello(HelloRequest {
                protocol_version: CONTROL_PROTOCOL_VERSION,
                provider_id: "mail".to_owned(),
            })),
            Err(ControlProtocolError::UnexpectedProvider("mail".to_owned()))
        );
    }

    #[test]
    fn handshake_returns_zulip_manifest() {
        let response = handle_control_request(ControlRequest::Hello(HelloRequest {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            provider_id: "zulip".to_owned(),
        }))
        .unwrap();
        let ControlResponse::Hello(response) = response else {
            panic!("expected hello response");
        };
        assert_eq!(response.manifest.provider_id.as_str(), "zulip");
        assert!(
            response
                .manifest
                .supports(makosh_provider_api::RuntimeTopology::SharedConnector)
        );
    }

    #[test]
    fn spool_deduplicates_and_fences_acknowledgements() {
        let mut spool = SpoolState::new(100).unwrap();
        let entry = SpoolEntry {
            stable_id: "observation-1".to_owned(),
            lease_epoch: 7,
            bytes: 80,
            acknowledged: false,
        };
        assert!(spool.insert(entry.clone()).unwrap());
        assert!(!spool.insert(entry).unwrap());
        assert_eq!(spool.health(), SpoolHealth::Degraded);
        assert!(matches!(
            spool.acknowledge("observation-1", 6),
            Err(SpoolError::StaleLeaseEpoch)
        ));
        spool.acknowledge("observation-1", 7).unwrap();
        assert!(matches!(
            spool.insert(SpoolEntry {
                stable_id: "observation-2".to_owned(),
                lease_epoch: 7,
                bytes: 21,
                acknowledged: false,
            }),
            Err(SpoolError::CapacityExceeded)
        ));
    }

    #[test]
    fn sqlite_spool_persists_and_fences_entries() {
        let path =
            std::env::temp_dir().join(format!("makosh-zulip-spool-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let spool = SqliteSpool::open(&path, 100).unwrap();
        let entry = SpoolEntry {
            stable_id: "command-1".to_owned(),
            lease_epoch: 3,
            bytes: 12,
            acknowledged: false,
        };
        assert!(spool.insert(&entry).unwrap());
        assert!(!spool.insert(&entry).unwrap());
        assert!(matches!(
            spool.acknowledge("command-1", 2),
            Err(SpoolDbError::StaleLeaseEpoch)
        ));
        spool.acknowledge("command-1", 3).unwrap();
        assert!(spool.remove_acknowledged("command-1").unwrap());
        assert!(!spool.remove_acknowledged("command-1").unwrap());
        drop(spool);
        let reopened = SqliteSpool::open(&path, 100).unwrap();
        assert!(reopened.insert(&entry).unwrap());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn subject_builders_reject_ambiguous_segments() {
        assert_eq!(
            command_subject("zulip", "account-1").unwrap(),
            "makosh.provider.commands.v1.zulip.account-1"
        );
        assert_eq!(
            observation_subject("zulip", "account-1", "message").unwrap(),
            "makosh.provider.observations.v1.zulip.account-1.message"
        );
        assert_eq!(
            command_subject("zulip.bad", "account-1"),
            Err(SubjectError::InvalidSegment)
        );
    }

    #[test]
    fn lifecycle_requires_explicit_start_and_drain() {
        let mut lifecycle = ConnectorLifecycle::default();
        assert_eq!(lifecycle.state(), ConnectorLifecycleState::Stopped);
        assert!(matches!(
            lifecycle.handle(ControlRequest::Drain),
            Err(ControlProtocolError::InvalidLifecycleTransition)
        ));
        lifecycle.handle(ControlRequest::Start).unwrap();
        assert_eq!(lifecycle.state(), ConnectorLifecycleState::Running);
        lifecycle.handle(ControlRequest::Drain).unwrap();
        assert_eq!(lifecycle.state(), ConnectorLifecycleState::Draining);
        let ControlResponse::Health { ready, draining } =
            lifecycle.handle(ControlRequest::Health).unwrap()
        else {
            panic!("expected health response");
        };
        assert!(!ready);
        assert!(draining);
        lifecycle.handle(ControlRequest::Stop).unwrap();
        assert_eq!(lifecycle.state(), ConnectorLifecycleState::Stopped);
    }

    #[test]
    fn auth_and_lease_control_requests_validate_without_exposing_secrets() {
        let auth = handle_control_request(ControlRequest::BeginAuth {
            account_id: "account-1".to_owned(),
        })
        .unwrap();
        assert_eq!(
            auth,
            ControlResponse::AuthStarted {
                account_id: "account-1".to_owned()
            }
        );
        assert_eq!(
            handle_control_request(ControlRequest::RenewCredentialLease {
                account_id: "account-1".to_owned(),
                epoch: 0,
            }),
            Err(ControlProtocolError::InvalidLeaseRequest)
        );
        assert_eq!(
            handle_control_request(ControlRequest::CompleteAuth {
                account_id: "account-1".to_owned(),
                callback_token: String::new(),
            }),
            Err(ControlProtocolError::InvalidAuthRequest)
        );
    }
}
