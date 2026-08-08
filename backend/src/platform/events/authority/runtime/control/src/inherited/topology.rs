//! Event Hub topology conversion and broker reconciliation inside the authority.

use std::os::unix::net::UnixStream;
use std::time::Duration;

use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, EventHubCredentialFenceV1, EventHubCredentialLeaseAdapterV1,
    EventHubTopologyPlanV1, JetStreamClient, StreamBudgetV1, StreamKindV1, StreamSpecV1,
};
use makosh_runtime_protocol::v1::{
    DurableEnvelopeKindV1, EventHubConsumerTopologyV1, EventHubStreamTopologyV1,
    EventsAuthorityRuntimeConfigurationV1, ReconcileEventsTopologyRequestV1,
};

use super::{
    handshake::EventsAuthorityRuntimeIdentityV1, vault_context,
    vault_route::InheritedVaultRoutePortV1,
};

const EVENT_HUB_INSTANCE_ID: &str = "event_hub_main";
const AUTHORITY_RUNTIME_INSTANCE_ID: &str = "events_authority_runtime";

pub(crate) fn reconcile(
    channel: &UnixStream,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
    request: ReconcileEventsTopologyRequestV1,
) -> Result<(), ()> {
    let topology = topology_plan(&request)?;
    let context = vault_context::from_configuration(configuration)?;
    let route = channel.try_clone().map_err(|_| ())?;
    let fence = credential_fence(identity, configuration)?;
    let mut leases =
        EventHubCredentialLeaseAdapterV1::new(InheritedVaultRoutePortV1::new(route), context);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| ())?;
    runtime.block_on(async {
        let credential = leases
            .resolve_event_hub_identity(&fence)
            .await
            .map_err(|_| ())?;
        let connection =
            JetStreamClient::connect_event_hub(&configuration.nats_endpoint, credential)
                .await
                .map_err(|_| ())?;
        connection.reconcile(&topology).await.map_err(|_| ())
    })
}

fn topology_plan(request: &ReconcileEventsTopologyRequestV1) -> Result<EventHubTopologyPlanV1, ()> {
    let streams = request
        .streams
        .iter()
        .map(stream_spec)
        .collect::<Result<Vec<_>, _>>()?;
    let consumers = request
        .consumers
        .iter()
        .map(consumer_spec)
        .collect::<Result<Vec<_>, _>>()?;
    EventHubTopologyPlanV1::new(streams, consumers).map_err(|_| ())
}

fn stream_spec(value: &EventHubStreamTopologyV1) -> Result<StreamSpecV1, ()> {
    let budget = StreamBudgetV1::new(
        i64::try_from(value.max_bytes).map_err(|_| ())?,
        Duration::from_millis(value.max_age_millis),
        usize::try_from(value.replicas).map_err(|_| ())?,
    )
    .map_err(|_| ())?;
    Ok(StreamSpecV1::new(stream_kind(value.envelope_kind)?, budget))
}

fn consumer_spec(value: &EventHubConsumerTopologyV1) -> Result<ConsumerSpecV1, ()> {
    let budget = ConsumerBudgetV1::new(
        i64::from(value.max_ack_pending),
        i64::from(value.max_deliver),
        Duration::from_millis(value.ack_wait_millis),
    )
    .map_err(|_| ())?;
    ConsumerSpecV1::new(
        stream_kind(value.envelope_kind)?,
        value.durable_name.clone(),
        value.filter_subject.clone(),
        budget,
    )
    .map_err(|_| ())
}

fn stream_kind(value: i32) -> Result<StreamKindV1, ()> {
    match DurableEnvelopeKindV1::try_from(value).ok() {
        Some(DurableEnvelopeKindV1::Command) => Ok(StreamKindV1::Command),
        Some(DurableEnvelopeKindV1::Event) => Ok(StreamKindV1::Event),
        Some(DurableEnvelopeKindV1::Observation) => Ok(StreamKindV1::Observation),
        Some(DurableEnvelopeKindV1::Result) => Ok(StreamKindV1::Result),
        Some(DurableEnvelopeKindV1::Ack) => Ok(StreamKindV1::Ack),
        _ => Err(()),
    }
}

fn credential_fence(
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<EventHubCredentialFenceV1, ()> {
    EventHubCredentialFenceV1::new(
        identity.registration_id().to_owned(),
        AUTHORITY_RUNTIME_INSTANCE_ID,
        EVENT_HUB_INSTANCE_ID,
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.event_hub_credential_revision,
        configuration.nats_username.clone(),
    )
    .map_err(|_| ())
}
