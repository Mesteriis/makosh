//! Bounded decisions for a due fire when its coordination key is occupied.

use makosh_scheduler_protocol::OverlapPolicyV1;

/// Decides whether a due occurrence starts, remains durable pending, or drops.
#[must_use]
pub const fn decide_due_overlap(
    policy: OverlapPolicyV1,
    active_runs: u16,
    pending_runs: u16,
) -> DueOverlapDecisionV1 {
    match policy {
        OverlapPolicyV1::Forbid => forbid(active_runs, pending_runs),
        OverlapPolicyV1::Queue { max_pending_runs } => {
            queue(active_runs, pending_runs, max_pending_runs)
        }
        OverlapPolicyV1::CoalesceLatest => coalesce(active_runs, pending_runs),
        OverlapPolicyV1::AllowBounded { max_parallelism } => {
            bounded(active_runs, pending_runs, max_parallelism)
        }
    }
}

const fn forbid(active_runs: u16, pending_runs: u16) -> DueOverlapDecisionV1 {
    if active_runs == 0 && pending_runs == 0 {
        DueOverlapDecisionV1::Start
    } else {
        DueOverlapDecisionV1::Drop
    }
}

const fn queue(active_runs: u16, pending_runs: u16, max_pending_runs: u16) -> DueOverlapDecisionV1 {
    if active_runs == 0 && pending_runs == 0 {
        DueOverlapDecisionV1::Start
    } else if pending_runs < max_pending_runs {
        DueOverlapDecisionV1::Enqueue
    } else {
        DueOverlapDecisionV1::Drop
    }
}

const fn coalesce(active_runs: u16, pending_runs: u16) -> DueOverlapDecisionV1 {
    if active_runs == 0 && pending_runs == 0 {
        DueOverlapDecisionV1::Start
    } else {
        DueOverlapDecisionV1::ReplacePending
    }
}

const fn bounded(
    active_runs: u16,
    pending_runs: u16,
    max_parallelism: u16,
) -> DueOverlapDecisionV1 {
    if pending_runs == 0 && active_runs < max_parallelism {
        DueOverlapDecisionV1::Start
    } else {
        DueOverlapDecisionV1::Drop
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DueOverlapDecisionV1 {
    Start,
    Enqueue,
    ReplacePending,
    Drop,
}
