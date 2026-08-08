//! Deterministic time source for conformance and owner tests.

use makosh_clock_protocol::{ClockDiscontinuityV1, ClockReadingV1, MonotonicMillisV1, UtcMillisV1};

#[derive(Clone, Debug)]
pub struct DeterministicClockV1 {
    wall_utc_millis: i64,
    monotonic_millis: u64,
    discontinuity: ClockDiscontinuityV1,
}

impl DeterministicClockV1 {
    #[must_use]
    pub const fn new(wall_utc_millis: i64) -> Self {
        Self {
            wall_utc_millis,
            monotonic_millis: 0,
            discontinuity: ClockDiscontinuityV1::Stable,
        }
    }

    #[must_use]
    pub const fn read(&self) -> ClockReadingV1 {
        ClockReadingV1::new(
            UtcMillisV1::new(self.wall_utc_millis),
            MonotonicMillisV1::new(self.monotonic_millis),
            self.discontinuity,
        )
    }

    pub fn advance(&mut self, elapsed_millis: u64) {
        self.monotonic_millis = self.monotonic_millis.saturating_add(elapsed_millis);
        self.wall_utc_millis = self.wall_utc_millis.saturating_add_unsigned(elapsed_millis);
        self.discontinuity = ClockDiscontinuityV1::Stable;
    }

    pub fn jump_wall(&mut self, wall_delta_millis: i64) {
        self.wall_utc_millis = self.wall_utc_millis.saturating_add(wall_delta_millis);
        self.discontinuity = if wall_delta_millis >= 0 {
            ClockDiscontinuityV1::WallClockJumpForward
        } else {
            ClockDiscontinuityV1::WallClockJumpBackward
        };
    }

    pub fn resume_after(&mut self, elapsed_millis: u64) {
        self.monotonic_millis = self.monotonic_millis.saturating_add(elapsed_millis);
        self.wall_utc_millis = self.wall_utc_millis.saturating_add_unsigned(elapsed_millis);
        self.discontinuity = ClockDiscontinuityV1::SuspendOrMonotonicGap;
    }
}
