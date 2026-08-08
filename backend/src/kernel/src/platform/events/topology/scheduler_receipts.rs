//! Scheduler receipt-consumer bindings derived only from approved Event Hub topology.

use std::collections::BTreeMap;

use makosh_runtime_protocol::v1::{
    SchedulerRuntimeReceiptConsumerBindingV1, SchedulerRuntimeReceiptKindV1,
};

use super::{EventConsumerPlanV1, EventTopologyPlanV1, subject::EventStreamKindV1};

const JOB_RECEIPT_CONTRACT: &str = "job_receipt";
const JOB_RECEIPT_MAJOR: u32 = 1;

pub(crate) fn scheduler_receipt_bindings(
    topology: &EventTopologyPlanV1,
    scheduler_registration_id: &str,
    scheduler_grant_epoch: u64,
) -> Result<Vec<SchedulerRuntimeReceiptConsumerBindingV1>, SchedulerReceiptTopologyErrorV1> {
    let mut pairs = BTreeMap::new();
    for consumer in topology.consumers().iter().filter(|consumer| {
        consumer.registration_id() == scheduler_registration_id
            && consumer.grant_epoch() == scheduler_grant_epoch
    }) {
        insert_consumer(&mut pairs, consumer)?;
    }
    finalize_pairs(pairs)
}

fn insert_consumer(
    pairs: &mut BTreeMap<String, ReceiptPairV1>,
    consumer: &EventConsumerPlanV1,
) -> Result<(), SchedulerReceiptTopologyErrorV1> {
    let subject = consumer.subject();
    if subject.contract() != JOB_RECEIPT_CONTRACT || subject.major() != JOB_RECEIPT_MAJOR {
        return Ok(());
    }
    let kind = receipt_kind(subject.kind()).ok_or(SchedulerReceiptTopologyErrorV1::InvalidKind)?;
    let binding = binding(kind, consumer);
    let pair = pairs.entry(subject.owner().to_owned()).or_default();
    let target = match kind {
        SchedulerRuntimeReceiptKindV1::Acceptance => &mut pair.acceptance,
        SchedulerRuntimeReceiptKindV1::Terminal => &mut pair.terminal,
        SchedulerRuntimeReceiptKindV1::Unspecified => {
            return Err(SchedulerReceiptTopologyErrorV1::InvalidKind);
        }
    };
    target
        .replace(binding)
        .is_none()
        .then_some(())
        .ok_or(SchedulerReceiptTopologyErrorV1::Duplicate)
}

fn finalize_pairs(
    pairs: BTreeMap<String, ReceiptPairV1>,
) -> Result<Vec<SchedulerRuntimeReceiptConsumerBindingV1>, SchedulerReceiptTopologyErrorV1> {
    let mut values = Vec::with_capacity(pairs.len().saturating_mul(2));
    for pair in pairs.into_values() {
        values.push(
            pair.acceptance
                .ok_or(SchedulerReceiptTopologyErrorV1::Incomplete)?,
        );
        values.push(
            pair.terminal
                .ok_or(SchedulerReceiptTopologyErrorV1::Incomplete)?,
        );
    }
    (!values.is_empty())
        .then_some(values)
        .ok_or(SchedulerReceiptTopologyErrorV1::Unavailable)
}

fn receipt_kind(kind: EventStreamKindV1) -> Option<SchedulerRuntimeReceiptKindV1> {
    match kind {
        EventStreamKindV1::Ack => Some(SchedulerRuntimeReceiptKindV1::Acceptance),
        EventStreamKindV1::Result => Some(SchedulerRuntimeReceiptKindV1::Terminal),
        _ => None,
    }
}

fn binding(
    kind: SchedulerRuntimeReceiptKindV1,
    consumer: &EventConsumerPlanV1,
) -> SchedulerRuntimeReceiptConsumerBindingV1 {
    let policy = consumer.delivery_policy();
    SchedulerRuntimeReceiptConsumerBindingV1 {
        kind: kind as i32,
        stream_name: stream_name(kind).to_owned(),
        durable_name: consumer.durable_name().to_owned(),
        filter_subject: consumer.subject().as_str(),
        ack_wait_millis: policy.ack_wait_millis(),
        max_deliver: u32::from(policy.max_deliver()),
        max_ack_pending: u32::from(consumer.max_in_flight()),
    }
}

fn stream_name(kind: SchedulerRuntimeReceiptKindV1) -> &'static str {
    match kind {
        SchedulerRuntimeReceiptKindV1::Acceptance => "MAKOSH_ACK_V1",
        SchedulerRuntimeReceiptKindV1::Terminal => "MAKOSH_RESULT_V1",
        SchedulerRuntimeReceiptKindV1::Unspecified => "",
    }
}

#[derive(Default)]
struct ReceiptPairV1 {
    acceptance: Option<SchedulerRuntimeReceiptConsumerBindingV1>,
    terminal: Option<SchedulerRuntimeReceiptConsumerBindingV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchedulerReceiptTopologyErrorV1 {
    Duplicate,
    Incomplete,
    InvalidKind,
    Unavailable,
}
