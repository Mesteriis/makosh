//! SQLite persistence for trusted Event Hub stream budgets.

use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

const MAX_STREAM_BYTES: u64 = 1_073_741_824;
const MAX_RETENTION_MILLIS: u64 = 90 * 24 * 60 * 60 * 1_000;

impl SqliteControlStore {
    pub fn record_platform_event_hub_topology(
        &self,
        topology: &PlatformEventHubTopologyV1,
    ) -> Result<(), StoreError> {
        valid_topology(topology)
            .then_some(())
            .ok_or(StoreError::InvalidPlatformEventHubTopology)?;
        let topology = topology.clone();
        self.with_connection(move |connection| {
            let transaction = connection.unchecked_transaction()?;
            let changed = transaction.execute(
                "INSERT INTO makosh_kernel_platform_event_hub_topology
                 (singleton, revision, nats_endpoint, nats_username, credential_revision)
                 VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(singleton) DO UPDATE SET revision=excluded.revision,
                   nats_endpoint=excluded.nats_endpoint, nats_username=excluded.nats_username,
                   credential_revision=excluded.credential_revision
                 WHERE excluded.revision = makosh_kernel_platform_event_hub_topology.revision + 1",
                params![
                    as_sql(topology.revision())?,
                    topology.nats_endpoint(),
                    topology.nats_username(),
                    as_sql(topology.credential_revision())?,
                ],
            )?;
            if changed != 1 {
                return Err(StoreError::PlatformEventHubTopologyRevisionConflict);
            }
            transaction.execute("DELETE FROM makosh_kernel_platform_event_stream_budget", [])?;
            for budget in topology.stream_budgets() {
                transaction.execute(
                    "INSERT INTO makosh_kernel_platform_event_stream_budget
                     (envelope_kind, max_bytes, max_age_millis, replicas)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        budget.envelope_kind().as_i64(),
                        as_sql(budget.max_bytes())?,
                        as_sql(budget.max_age_millis())?,
                        i64::from(budget.replicas()),
                    ],
                )?;
            }
            transaction.commit().map_err(StoreError::from)
        })
    }

    pub fn platform_event_hub_topology(
        &self,
    ) -> Result<Option<PlatformEventHubTopologyV1>, StoreError> {
        self.with_connection(move |connection| {
            let row = connection
                .query_row(
                    "SELECT revision, nats_endpoint, nats_username, credential_revision
                     FROM makosh_kernel_platform_event_hub_topology WHERE singleton=1",
                    [],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    },
                )
                .optional()?;
            row.map(|row| read_topology(connection, row)).transpose()
        })
    }
}

fn read_topology(
    connection: &rusqlite::Connection,
    row: (i64, String, String, i64),
) -> Result<PlatformEventHubTopologyV1, StoreError> {
    let (revision, nats_endpoint, nats_username, credential_revision) = row;
    let mut statement = connection.prepare(
        "SELECT envelope_kind, max_bytes, max_age_millis, replicas
         FROM makosh_kernel_platform_event_stream_budget ORDER BY envelope_kind",
    )?;
    let budgets = statement
        .query_map([], |row| {
            let kind = ModuleEventEnvelopeKindV1::from_i64(row.get(0)?)
                .ok_or(rusqlite::Error::IntegralValueOutOfRange(0, 0))?;
            Ok(PlatformEventStreamBudgetV1::new(
                kind,
                as_u64(row.get(1)?, 1)?,
                as_u64(row.get(2)?, 2)?,
                u8::try_from(row.get::<_, i64>(3)?)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 0))?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let topology = PlatformEventHubTopologyV1::new(
        as_u64(revision, 0)?,
        nats_endpoint,
        nats_username,
        as_u64(credential_revision, 3)?,
        budgets,
    );
    valid_topology(&topology)
        .then_some(topology)
        .ok_or(StoreError::InvalidPlatformEventHubTopology)
}

fn valid_topology(topology: &PlatformEventHubTopologyV1) -> bool {
    topology.revision() > 0
        && valid_endpoint(topology.nats_endpoint())
        && valid_identity(topology.nats_username())
        && topology.credential_revision() > 0
        && topology.stream_budgets().len() == 5
        && topology.stream_budgets().iter().all(valid_budget)
        && topology
            .stream_budgets()
            .iter()
            .map(|budget| budget.envelope_kind().as_i64())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == 5
}

fn valid_endpoint(value: &str) -> bool {
    value.starts_with("nats://") && value.len() <= 256 && !value.contains(['@', '?', '#', ' '])
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_budget(budget: &PlatformEventStreamBudgetV1) -> bool {
    (1..=MAX_STREAM_BYTES).contains(&budget.max_bytes())
        && (1..=MAX_RETENTION_MILLIS).contains(&budget.max_age_millis())
        && budget.replicas() == 1
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformEventHubTopology)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
