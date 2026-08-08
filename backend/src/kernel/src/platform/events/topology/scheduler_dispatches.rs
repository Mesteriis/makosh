//! Scheduler command-publisher bindings derived only from approved Event Hub topology.

use std::collections::BTreeSet;

use makosh_runtime_protocol::v1::SchedulerRuntimeDispatchPublisherBindingV1;

use super::{EventTopologyPlanV1, subject::EventStreamKindV1};

pub(crate) fn scheduler_dispatch_bindings(
    topology: &EventTopologyPlanV1,
    scheduler_registration_id: &str,
    scheduler_grant_epoch: u64,
) -> Result<Vec<SchedulerRuntimeDispatchPublisherBindingV1>, SchedulerDispatchTopologyErrorV1> {
    let mut subjects = BTreeSet::new();
    for publisher in topology.publishers().iter().filter(|publisher| {
        publisher.registration_id() == scheduler_registration_id
            && publisher.grant_epoch() == scheduler_grant_epoch
    }) {
        if publisher.subject().kind() == EventStreamKindV1::Command {
            subjects.insert(publisher.subject().as_str());
        }
    }
    (!subjects.is_empty())
        .then_some(
            subjects
                .into_iter()
                .map(|subject| SchedulerRuntimeDispatchPublisherBindingV1 { subject })
                .collect(),
        )
        .ok_or(SchedulerDispatchTopologyErrorV1::Unavailable)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchedulerDispatchTopologyErrorV1 {
    Unavailable,
}
