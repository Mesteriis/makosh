//! Generic managed-runtime pull-consumer bindings derived from approved topology.

use makosh_runtime_protocol::v1::ManagedRuntimeEventConsumerBindingV1;

use super::{
    EventConsumerPlanV1, EventPublisherPermitPlanV1, EventTopologyPlanV1,
    subject::EventStreamKindV1,
};

pub(crate) fn managed_runtime_publish_subjects(
    topology: &EventTopologyPlanV1,
    registration_id: &str,
    grant_epoch: u64,
) -> Vec<String> {
    let mut values = topology
        .publishers()
        .iter()
        .filter(|publisher| {
            publisher.registration_id() == registration_id && publisher.grant_epoch() == grant_epoch
        })
        .map(EventPublisherPermitPlanV1::subject)
        .map(|subject| subject.as_str())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

pub(crate) fn managed_runtime_consumer_bindings(
    topology: &EventTopologyPlanV1,
    registration_id: &str,
    grant_epoch: u64,
) -> Result<Vec<ManagedRuntimeEventConsumerBindingV1>, ManagedRuntimeConsumerBindingErrorV1> {
    let mut values = topology
        .consumers()
        .iter()
        .filter(|consumer| {
            consumer.registration_id() == registration_id && consumer.grant_epoch() == grant_epoch
        })
        .map(binding)
        .collect::<Result<Vec<_>, _>>()?;
    values.sort_by(|left, right| left.durable_name.cmp(&right.durable_name));
    values
        .windows(2)
        .all(|pair| pair[0].durable_name != pair[1].durable_name)
        .then_some(values)
        .ok_or(ManagedRuntimeConsumerBindingErrorV1::Duplicate)
}

fn binding(
    consumer: &EventConsumerPlanV1,
) -> Result<ManagedRuntimeEventConsumerBindingV1, ManagedRuntimeConsumerBindingErrorV1> {
    let policy = consumer.delivery_policy();
    Ok(ManagedRuntimeEventConsumerBindingV1 {
        stream_name: stream_name(consumer.subject().kind())?.to_owned(),
        durable_name: consumer.durable_name().to_owned(),
        filter_subject: consumer.subject().as_str(),
        ack_wait_millis: policy.ack_wait_millis(),
        max_deliver: u32::from(policy.max_deliver()),
        max_ack_pending: u32::from(consumer.max_in_flight()),
        contract: Some(consumer.contract().clone()),
    })
}

fn stream_name(
    kind: EventStreamKindV1,
) -> Result<&'static str, ManagedRuntimeConsumerBindingErrorV1> {
    match kind {
        EventStreamKindV1::Command => Ok("MAKOSH_COMMAND_V1"),
        EventStreamKindV1::Event => Ok("MAKOSH_EVENT_V1"),
        EventStreamKindV1::Observation => Ok("MAKOSH_OBSERVATION_V1"),
        EventStreamKindV1::Result => Ok("MAKOSH_RESULT_V1"),
        EventStreamKindV1::Ack => Ok("MAKOSH_ACK_V1"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManagedRuntimeConsumerBindingErrorV1 {
    Duplicate,
}
