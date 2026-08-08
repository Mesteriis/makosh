//! Owner-authorized desired Event Hub topology changes.

use makosh_gateway_protocol::v1::{
    ConfigurePlatformEventHubTopologyRequestV1, ConfigurePlatformEventHubTopologyResponseV1,
    EventHubStreamBudgetV1, ReconcilePlatformEventHubTopologyRequestV1,
    ReconcilePlatformEventHubTopologyResponseV1,
};
use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::super::{OwnerControlSessions, OwnerResult};
use crate::platform::events::reconciliation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(super) fn configure(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ConfigurePlatformEventHubTopologyRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let topology = PlatformEventHubTopologyV1::new(
            next_revision(store)?,
            request.nats_endpoint,
            request.nats_username,
            request.credential_revision,
            request
                .stream_budgets
                .into_iter()
                .map(stream_budget)
                .collect::<Result<Vec<_>, _>>()?,
        );
        store
            .record_platform_event_hub_topology(&topology)
            .map_err(|_| "Event Hub topology cannot be recorded".to_owned())?;
        Ok(topology)
    })()
    .map(|topology| {
        OwnerResult::ConfigurePlatformEventHubTopology(
            ConfigurePlatformEventHubTopologyResponseV1 {
                topology_revision: topology.revision(),
                stream_count: u32::try_from(topology.stream_budgets().len()).unwrap_or_default(),
            },
        )
    })
}

pub(super) fn reconcile(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ReconcilePlatformEventHubTopologyRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        reconciliation::apply(store, &supervisor.relay_port())
    })()
    .map(|(topology_revision, stream_count, consumer_count)| {
        OwnerResult::ReconcilePlatformEventHubTopology(
            ReconcilePlatformEventHubTopologyResponseV1 {
                topology_revision,
                stream_count,
                consumer_count,
            },
        )
    })
}

fn stream_budget(value: EventHubStreamBudgetV1) -> Result<PlatformEventStreamBudgetV1, String> {
    let kind = ModuleEventEnvelopeKindV1::from_i64(i64::from(value.envelope_kind))
        .ok_or_else(|| "Event Hub stream budget is invalid".to_owned())?;
    let replicas = u8::try_from(value.replicas)
        .map_err(|_| "Event Hub stream budget is invalid".to_owned())?;
    Ok(PlatformEventStreamBudgetV1::new(
        kind,
        value.max_bytes,
        value.max_age_millis,
        replicas,
    ))
}

fn next_revision(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_event_hub_topology()
        .map_err(|_| "Event Hub topology is unavailable".to_owned())?
        .map_or(Ok(1), |current| {
            current
                .revision()
                .checked_add(1)
                .ok_or_else(|| "Event Hub topology revision overflowed".to_owned())
        })
}
