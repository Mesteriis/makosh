use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{SchedulePolicyV1, ScheduleTriggerV1};

use super::interval::plan_interval;

/// Plans only time occurrences. It neither claims slots nor runs owner code.
pub fn plan_due(
    policy: &SchedulePolicyV1,
    next_due_at: UtcMillisV1,
    now: UtcMillisV1,
) -> Result<DueSchedulePlanV1, SchedulePlanErrorV1> {
    if next_due_at > now {
        return Ok(DueSchedulePlanV1::not_due(next_due_at));
    }
    match policy.trigger() {
        ScheduleTriggerV1::At { .. } => Ok(DueSchedulePlanV1::dispatch(
            vec![next_due_at],
            ScheduleContinuationV1::Dormant,
        )),
        ScheduleTriggerV1::FixedInterval { interval_millis } => {
            plan_interval(next_due_at, now, *interval_millis, policy.misfire())
        }
        ScheduleTriggerV1::FixedDelay { delay_millis } => Ok(DueSchedulePlanV1::dispatch(
            vec![next_due_at],
            ScheduleContinuationV1::AfterTerminalDelay(*delay_millis),
        )),
        ScheduleTriggerV1::Cron { .. } => Err(SchedulePlanErrorV1::CronNotImplemented),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DueSchedulePlanV1 {
    occurrences: Vec<UtcMillisV1>,
    continuation: ScheduleContinuationV1,
}

impl DueSchedulePlanV1 {
    pub(super) fn not_due(next_due_at: UtcMillisV1) -> Self {
        Self {
            occurrences: Vec::new(),
            continuation: ScheduleContinuationV1::At(next_due_at),
        }
    }

    pub(super) fn dispatch(
        occurrences: Vec<UtcMillisV1>,
        continuation: ScheduleContinuationV1,
    ) -> Self {
        Self {
            occurrences,
            continuation,
        }
    }

    #[must_use]
    pub fn occurrences(&self) -> &[UtcMillisV1] {
        &self.occurrences
    }

    #[must_use]
    pub const fn continuation(&self) -> ScheduleContinuationV1 {
        self.continuation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleContinuationV1 {
    At(UtcMillisV1),
    AfterTerminalDelay(u64),
    Dormant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulePlanErrorV1 {
    CronNotImplemented,
    TimeOverflow,
}
