use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{ConcurrencyKeyV1, SchedulePolicyV1, ScheduleRevisionV1};

use crate::SchedulerRunClaimV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerPendingFireV1 {
    claim: SchedulerRunClaimV1,
    recorded_at: UtcMillisV1,
}

impl SchedulerPendingFireV1 {
    pub fn new(
        claim: SchedulerRunClaimV1,
        recorded_at: UtcMillisV1,
    ) -> Result<Self, SchedulerPendingFireErrorV1> {
        (claim.scheduled_for() <= recorded_at)
            .then_some(Self { claim, recorded_at })
            .ok_or(SchedulerPendingFireErrorV1::Invalid)
    }

    #[must_use]
    pub fn claim(&self) -> &SchedulerRunClaimV1 {
        &self.claim
    }

    #[must_use]
    pub const fn recorded_at(&self) -> UtcMillisV1 {
        self.recorded_at
    }

    #[must_use]
    pub fn concurrency_key(&self) -> &ConcurrencyKeyV1 {
        self.claim.concurrency_key()
    }

    #[must_use]
    pub const fn schedule_revision(&self) -> ScheduleRevisionV1 {
        self.claim.schedule_revision()
    }

    #[must_use]
    pub fn policy(&self) -> &SchedulePolicyV1 {
        self.claim.policy()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerPendingFireOutcomeV1 {
    Queued,
    AlreadyQueued,
    Coalesced,
    Dropped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerPendingFireErrorV1 {
    Invalid,
    Stale,
    Unavailable,
}
