use makosh_events_protocol::delivery::OutboxRecordV1;

use crate::SchedulerRunClaimV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedulerDispatchClaimV1 {
    claim: SchedulerRunClaimV1,
    record: OutboxRecordV1,
}

impl SchedulerDispatchClaimV1 {
    pub fn new(
        claim: SchedulerRunClaimV1,
        record: OutboxRecordV1,
    ) -> Result<Self, SchedulerDispatchClaimErrorV1> {
        (record.message_id() == &claim.dispatch_message_id())
            .then_some(Self { claim, record })
            .ok_or(SchedulerDispatchClaimErrorV1::MessageIdMismatch)
    }

    #[must_use]
    pub fn claim(&self) -> &SchedulerRunClaimV1 {
        &self.claim
    }

    #[must_use]
    pub fn record(&self) -> &OutboxRecordV1 {
        &self.record
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerDispatchClaimErrorV1 {
    MessageIdMismatch,
    ClaimDenied,
    Unavailable,
}
