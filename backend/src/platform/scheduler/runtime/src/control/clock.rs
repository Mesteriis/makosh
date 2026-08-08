//! Local Scheduler wall-clock read with explicit discontinuity fencing.

use std::{
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use makosh_clock_protocol::{
    ClockDiscontinuityV1, ClockPolicyV1, ClockReadingV1, MonotonicMillisV1, UtcMillisV1,
};

pub(super) struct SchedulerSystemClockV1 {
    origin: Instant,
    policy: ClockPolicyV1,
    previous: Mutex<Option<ClockReadingV1>>,
}

impl SchedulerSystemClockV1 {
    pub(super) fn new(policy: ClockPolicyV1) -> Self {
        Self {
            origin: Instant::now(),
            policy,
            previous: Mutex::new(None),
        }
    }

    pub(super) fn read(&self) -> Result<ClockReadingV1, ()> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ())?;
        let wall = i64::try_from(duration.as_millis()).map_err(|_| ())?;
        let monotonic = u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX);
        let current = ClockReadingV1::new(
            UtcMillisV1::new(wall),
            MonotonicMillisV1::new(monotonic),
            ClockDiscontinuityV1::Stable,
        );
        let mut previous = self.previous.lock().map_err(|_| ())?;
        let classified =
            previous.map_or(current, |previous| classify(previous, current, self.policy));
        *previous = Some(classified);
        Ok(classified)
    }
}

fn classify(
    previous: ClockReadingV1,
    current: ClockReadingV1,
    policy: ClockPolicyV1,
) -> ClockReadingV1 {
    let monotonic_delta = current
        .monotonic()
        .value()
        .saturating_sub(previous.monotonic().value());
    let wall_delta = current
        .wall_utc()
        .value()
        .saturating_sub(previous.wall_utc().value());
    let expected = i64::try_from(monotonic_delta).unwrap_or(i64::MAX);
    let drift = wall_delta.saturating_sub(expected);
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
