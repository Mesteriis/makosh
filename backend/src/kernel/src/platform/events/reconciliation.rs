//! Kernel relay for one owner-authorized Event Hub reconciliation.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        DurableEnvelopeKindV1, EventHubConsumerTopologyV1, EventHubStreamTopologyV1,
        EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
        ReconcileEventsTopologyRequestV1, events_authority_runtime_control_request_v1::Operation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::validate_events_authority_runtime_control_response,
};
use prost::Message;

use crate::platform::events::{
    authority::{binding::EVENTS_AUTHORITY_PROCESS_ID, status},
    catalog, topology,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub(crate) fn apply(
    store: &SqliteControlStore,
    relay: &dyn ManagedRuntimeRelay,
) -> Result<(u64, u32, u32), String> {
    let configuration = status::current_topology(store)?;
    let contracts = catalog::resolve_contracts(store)?;
    let plan = topology::plan(&contracts, &configuration)?;
    let request = request(configuration.revision(), &plan)?;
    apply_recovery_snapshot(&request.encode_to_vec(), relay)
}

pub(crate) fn apply_recovery_snapshot(
    bytes: &[u8],
    relay: &dyn ManagedRuntimeRelay,
) -> Result<(u64, u32, u32), String> {
    use makosh_runtime_protocol::validation::events_authority::validate_events_authority_runtime_control_request;

    let request = EventsAuthorityRuntimeControlRequestV1::decode(bytes)
        .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
    validate_events_authority_runtime_control_request(&request)
        .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
    let revision = request_revision(&request)?;
    let response = relay.relay(EVENTS_AUTHORITY_PROCESS_ID, bytes.to_vec())?;
    parse_response(
        response.as_slice(),
        revision,
        request_stream_count(&request)?,
        request_consumer_count(&request)?,
    )
}

pub(crate) fn recovery_snapshot(store: &SqliteControlStore) -> Result<Vec<u8>, String> {
    let configuration = status::current_topology(store)?;
    let contracts = catalog::resolve_contracts(store)?;
    let plan = topology::plan(&contracts, &configuration)?;
    Ok(request(configuration.revision(), &plan)?.encode_to_vec())
}

pub(crate) fn validate_recovery_snapshot(
    store: &SqliteControlStore,
    bytes: &[u8],
) -> Result<(), String> {
    use makosh_runtime_protocol::validation::events_authority::validate_events_authority_runtime_control_request;

    let snapshot = EventsAuthorityRuntimeControlRequestV1::decode(bytes)
        .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
    validate_events_authority_runtime_control_request(&snapshot)
        .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
    (recovery_snapshot(store)? == bytes)
        .then_some(())
        .ok_or_else(|| "Event Hub recovery topology does not match restored authority".to_owned())
}

fn request(
    revision: u64,
    plan: &topology::EventTopologyPlanV1,
) -> Result<EventsAuthorityRuntimeControlRequestV1, String> {
    let streams = plan.streams().iter().map(stream).collect::<Vec<_>>();
    let consumers = plan.consumers().iter().map(consumer).collect::<Vec<_>>();
    (!streams.is_empty())
        .then_some(EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(Operation::ReconcileTopology(
                ReconcileEventsTopologyRequestV1 {
                    topology_revision: revision,
                    streams,
                    consumers,
                },
            )),
        })
        .ok_or_else(|| "Event Hub topology has no admitted streams".to_owned())
}

fn stream(value: &topology::plan::EventStreamPlanV1) -> EventHubStreamTopologyV1 {
    EventHubStreamTopologyV1 {
        envelope_kind: envelope_kind(value.kind()) as i32,
        max_bytes: value.max_bytes(),
        max_age_millis: value.max_age_millis(),
        replicas: u32::from(value.replicas()),
    }
}

fn consumer(value: &topology::EventConsumerPlanV1) -> EventHubConsumerTopologyV1 {
    let policy = value.delivery_policy();
    EventHubConsumerTopologyV1 {
        envelope_kind: envelope_kind(value.subject().kind()) as i32,
        durable_name: value.durable_name().to_owned(),
        filter_subject: value.subject().as_str(),
        max_ack_pending: u32::from(value.max_in_flight()),
        max_deliver: u32::from(policy.max_deliver()),
        ack_wait_millis: u64::from(policy.ack_wait_millis()),
    }
}

fn envelope_kind(value: topology::subject::EventStreamKindV1) -> DurableEnvelopeKindV1 {
    match value {
        topology::subject::EventStreamKindV1::Command => DurableEnvelopeKindV1::Command,
        topology::subject::EventStreamKindV1::Event => DurableEnvelopeKindV1::Event,
        topology::subject::EventStreamKindV1::Observation => DurableEnvelopeKindV1::Observation,
        topology::subject::EventStreamKindV1::Result => DurableEnvelopeKindV1::Result,
        topology::subject::EventStreamKindV1::Ack => DurableEnvelopeKindV1::Ack,
    }
}

fn parse_response(
    bytes: &[u8],
    expected_revision: u64,
    expected_streams: u32,
    expected_consumers: u32,
) -> Result<(u64, u32, u32), String> {
    let response = EventsAuthorityRuntimeControlResponseV1::decode(bytes)
        .map_err(|_| "Events authority reconciliation response is invalid".to_owned())?;
    validate_events_authority_runtime_control_response(&response)
        .map_err(|_| "Events authority reconciliation response is invalid".to_owned())?;
    if !response.error_code.is_empty() {
        return Err(format!(
            "Events authority reconciliation was denied: {}",
            response.error_code
        ));
    }
    let Some(ResponseResult::TopologyReconciled(value)) = response.result else {
        return Err("Events authority reconciliation response is unavailable".to_owned());
    };
    (value.topology_revision == expected_revision
        && value.stream_count == expected_streams
        && value.consumer_count == expected_consumers)
        .then_some((
            value.topology_revision,
            value.stream_count,
            value.consumer_count,
        ))
        .ok_or_else(|| "Events authority reconciliation response is stale".to_owned())
}

fn request_stream_count(request: &EventsAuthorityRuntimeControlRequestV1) -> Result<u32, String> {
    match &request.operation {
        Some(Operation::ReconcileTopology(value)) => u32::try_from(value.streams.len())
            .map_err(|_| "Event Hub topology is oversized".to_owned()),
        _ => Err("Event Hub reconciliation request is unavailable".to_owned()),
    }
}

fn request_consumer_count(request: &EventsAuthorityRuntimeControlRequestV1) -> Result<u32, String> {
    match &request.operation {
        Some(Operation::ReconcileTopology(value)) => u32::try_from(value.consumers.len())
            .map_err(|_| "Event Hub topology is oversized".to_owned()),
        _ => Err("Event Hub reconciliation request is unavailable".to_owned()),
    }
}

fn request_revision(request: &EventsAuthorityRuntimeControlRequestV1) -> Result<u64, String> {
    match &request.operation {
        Some(Operation::ReconcileTopology(value)) => Ok(value.topology_revision),
        _ => Err("Event Hub reconciliation request is unavailable".to_owned()),
    }
}
