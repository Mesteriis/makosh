use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::MisfirePolicyV1;

use super::{DueSchedulePlanV1, ScheduleContinuationV1, SchedulePlanErrorV1};

pub(super) fn plan_interval(
    due: UtcMillisV1,
    now: UtcMillisV1,
    interval_millis: u64,
    misfire: MisfirePolicyV1,
) -> Result<DueSchedulePlanV1, SchedulePlanErrorV1> {
    let interval = i64::try_from(interval_millis).map_err(|_| SchedulePlanErrorV1::TimeOverflow)?;
    let elapsed = now.value().saturating_sub(due.value());
    let missed =
        u64::try_from(elapsed / interval).map_err(|_| SchedulePlanErrorV1::TimeOverflow)?;
    let next_due = advance(due, missed.saturating_add(1), interval)?;
    let occurrences = match misfire {
        MisfirePolicyV1::Skip if missed > 0 => Vec::new(),
        MisfirePolicyV1::Skip | MisfirePolicyV1::FireOnce => vec![advance(due, missed, interval)?],
        MisfirePolicyV1::CatchUpBounded { max_runs } => {
            bounded_occurrences(due, missed, interval, max_runs)?
        }
    };
    Ok(DueSchedulePlanV1::dispatch(
        occurrences,
        ScheduleContinuationV1::At(next_due),
    ))
}

fn bounded_occurrences(
    due: UtcMillisV1,
    missed: u64,
    interval: i64,
    max_runs: u16,
) -> Result<Vec<UtcMillisV1>, SchedulePlanErrorV1> {
    let count = u64::from(max_runs).min(missed.saturating_add(1));
    let first = missed.saturating_add(1).saturating_sub(count);
    (first..=missed)
        .map(|offset| advance(due, offset, interval))
        .collect()
}

fn advance(
    due: UtcMillisV1,
    count: u64,
    interval: i64,
) -> Result<UtcMillisV1, SchedulePlanErrorV1> {
    let count = i64::try_from(count).map_err(|_| SchedulePlanErrorV1::TimeOverflow)?;
    due.value()
        .checked_add(
            interval
                .checked_mul(count)
                .ok_or(SchedulePlanErrorV1::TimeOverflow)?,
        )
        .map(UtcMillisV1::new)
        .ok_or(SchedulePlanErrorV1::TimeOverflow)
}
