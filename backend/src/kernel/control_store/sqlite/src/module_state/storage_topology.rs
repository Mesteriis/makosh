//! SQLite persistence for the desired, non-secret Storage platform topology.

use makosh_kernel_control_store::{
    PlatformStorageEndpointV1, PlatformStorageTopology, PlatformStorageTopologyInputV1,
    StorageDeploymentProfileV1,
};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn record_platform_storage_topology(
        &self,
        topology: &PlatformStorageTopology,
    ) -> Result<(), StoreError> {
        if !valid_topology(topology) {
            return Err(StoreError::InvalidPlatformStorageTopology);
        }
        let topology = topology.clone();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_platform_storage_topology (singleton, revision, storage_generation, storage_instance_id, database_id, deployment_profile, postgres_host, postgres_port, pgbouncer_host, pgbouncer_port, pgbouncer_backend_host, pgbouncer_backend_port, postgres_artifact_sha256, pgbouncer_artifact_sha256) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) ON CONFLICT(singleton) DO UPDATE SET revision=excluded.revision, storage_generation=excluded.storage_generation, storage_instance_id=excluded.storage_instance_id, database_id=excluded.database_id, deployment_profile=excluded.deployment_profile, postgres_host=excluded.postgres_host, postgres_port=excluded.postgres_port, pgbouncer_host=excluded.pgbouncer_host, pgbouncer_port=excluded.pgbouncer_port, pgbouncer_backend_host=excluded.pgbouncer_backend_host, pgbouncer_backend_port=excluded.pgbouncer_backend_port, postgres_artifact_sha256=excluded.postgres_artifact_sha256, pgbouncer_artifact_sha256=excluded.pgbouncer_artifact_sha256 WHERE excluded.revision = makosh_kernel_platform_storage_topology.revision + 1 AND excluded.storage_generation > makosh_kernel_platform_storage_topology.storage_generation",
                params![as_sql(topology.revision())?, as_sql(topology.storage_generation())?, topology.storage_instance_id(), topology.database_id(), topology.deployment_profile().as_str(), topology.postgres_endpoint().host(), i64::from(topology.postgres_endpoint().port()), topology.pgbouncer_endpoint().host(), i64::from(topology.pgbouncer_endpoint().port()), topology.pgbouncer_backend_endpoint().host(), i64::from(topology.pgbouncer_backend_endpoint().port()), topology.postgres_artifact_sha256().as_slice(), topology.pgbouncer_artifact_sha256().as_slice()],
            )?;
            if changed == 1 {
                Ok(())
            } else {
                Err(StoreError::PlatformStorageTopologyRevisionConflict)
            }
        })
    }

    pub fn platform_storage_topology(&self) -> Result<Option<PlatformStorageTopology>, StoreError> {
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT revision, storage_generation, storage_instance_id, database_id, deployment_profile, postgres_host, postgres_port, pgbouncer_host, pgbouncer_port, pgbouncer_backend_host, pgbouncer_backend_port, postgres_artifact_sha256, pgbouncer_artifact_sha256 FROM makosh_kernel_platform_storage_topology WHERE singleton = 1",
                    [],
                    decode_topology,
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
}

fn decode_topology(row: &rusqlite::Row<'_>) -> Result<PlatformStorageTopology, rusqlite::Error> {
    let profile: String = row.get(4)?;
    let postgres_digest: Vec<u8> = row.get(11)?;
    let pgbouncer_digest: Vec<u8> = row.get(12)?;
    let profile = StorageDeploymentProfileV1::parse(&profile).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(
            4,
            "deployment_profile".into(),
            rusqlite::types::Type::Text,
        )
    })?;
    Ok(
        PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
            revision: as_u64(row.get(0)?, 0)?,
            storage_generation: as_u64(row.get(1)?, 1)?,
            storage_instance_id: row.get(2)?,
            database_id: row.get(3)?,
            deployment_profile: profile,
            postgres_endpoint: endpoint(row.get(5)?, row.get(6)?, 5, 6)?,
            pgbouncer_endpoint: endpoint(row.get(7)?, row.get(8)?, 7, 8)?,
            postgres_artifact_sha256: postgres_digest
                .try_into()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(11, 32))?,
            pgbouncer_artifact_sha256: pgbouncer_digest
                .try_into()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(12, 32))?,
        })
        .with_pgbouncer_backend_endpoint(endpoint(row.get(9)?, row.get(10)?, 9, 10)?),
    )
}

fn valid_topology(value: &PlatformStorageTopology) -> bool {
    value.revision() > 0
        && value.storage_generation() > 0
        && valid_identity_token(value.storage_instance_id())
        && valid_identity_token(value.database_id())
        && valid_endpoint(value.postgres_endpoint())
        && valid_endpoint(value.pgbouncer_endpoint())
        && valid_endpoint(value.pgbouncer_backend_endpoint())
        && value
            .postgres_artifact_sha256()
            .iter()
            .any(|byte| *byte != 0)
        && value
            .pgbouncer_artifact_sha256()
            .iter()
            .any(|byte| *byte != 0)
}

fn endpoint(
    host: String,
    port: i64,
    host_index: usize,
    port_index: usize,
) -> Result<PlatformStorageEndpointV1, rusqlite::Error> {
    let port = u16::try_from(port)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(rusqlite::Error::IntegralValueOutOfRange(port_index, 0))?;
    let endpoint = PlatformStorageEndpointV1::new(host, port);
    valid_endpoint(&endpoint)
        .then_some(endpoint)
        .ok_or_else(|| {
            rusqlite::Error::InvalidColumnType(
                host_index,
                "storage endpoint".into(),
                rusqlite::types::Type::Text,
            )
        })
}

fn valid_endpoint(value: &PlatformStorageEndpointV1) -> bool {
    value.port() > 0
        && !value.host().is_empty()
        && value.host().len() <= 253
        && value.host().bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_uppercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b':')
        })
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformStorageTopology)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
