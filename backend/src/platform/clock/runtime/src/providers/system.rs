//! System clock adapter with explicit discontinuity classification.

use std::{
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use makosh_clock_protocol::{
    ClockDiscontinuityV1, ClockPolicyV1, ClockReadingV1, MonotonicMillisV1, UtcMillisV1,
};

#[derive(Debug)]
pub struct SystemClockV1 {
    origin: Instant,
    policy: ClockPolicyV1,
    previous: Mutex<Option<ClockReadingV1>>,
}

impl SystemClockV1 {
    #[must_use]
    pub fn new(policy: ClockPolicyV1) -> Self {
        Self {
            origin: Instant::now(),
            policy,
            previous: Mutex::new(None),
        }
    }

    pub fn read(&self) -> Result<ClockReadingV1, SystemClockErrorV1> {
        let reading = ClockReadingV1::new(
            UtcMillisV1::new(system_utc_millis()?),
            MonotonicMillisV1::new(elapsed_millis(self.origin)),
            ClockDiscontinuityV1::Stable,
        );
        let mut previous = self
            .previous
            .lock()
            .map_err(|_| SystemClockErrorV1::StatePoisoned)?;
        let classified = classify(*previous, reading, self.policy);
        *previous = Some(classified);
        Ok(classified)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemClockErrorV1 {
    BeforeUnixEpoch,
    MillisecondOverflow,
    StatePoisoned,
}

fn system_utc_millis() -> Result<i64, SystemClockErrorV1> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SystemClockErrorV1::BeforeUnixEpoch)?;
    i64::try_from(duration.as_millis()).map_err(|_| SystemClockErrorV1::MillisecondOverflow)
}

fn elapsed_millis(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn classify(
    previous: Option<ClockReadingV1>,
    current: ClockReadingV1,
    policy: ClockPolicyV1,
) -> ClockReadingV1 {
    let Some(previous) = previous else {
        return current;
    };
    let monotonic_delta = current
        .monotonic()
        .value()
        .saturating_sub(previous.monotonic().value());
    let wall_delta = current
        .wall_utc()
        .value()
        .saturating_sub(previous.wall_utc().value());
    let expected_wall_delta = i64::try_from(monotonic_delta).unwrap_or(i64::MAX);
    let drift = wall_delta.saturating_sub(expected_wall_delta);
    let discontinuity = if monotonic_delta >= policy.suspend_gap_millis() {
        ClockDiscontinuityV1::SuspendOrMonotonicGap
    } else if drift >= i64::try_from(policy.jump_tolerance_millis()).unwrap_or(i64::MAX) {
        ClockDiscontinuityV1::WallClockJumpForward
    } else if drift <= -i64::try_from(policy.jump_tolerance_millis()).unwrap_or(i64::MAX) {
        ClockDiscontinuityV1::WallClockJumpBackward
    } else {
        ClockDiscontinuityV1::Stable
    };
    ClockReadingV1::new(current.wall_utc(), current.monotonic(), discontinuity)
}
