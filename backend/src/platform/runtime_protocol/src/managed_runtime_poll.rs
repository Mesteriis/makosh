//! Bounded idle backoff for independently managed runtime process loops.
//!
//! A runtime must drain immediately while work is available, but continuously
//! querying an empty owner store every fixed tick wastes CPU once dozens of
//! isolated processes are online. This helper keeps the active path immediate
//! and bounds idle control latency without introducing a shared scheduler.

use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedRuntimePollBackoffV1 {
    minimum: Duration,
    maximum: Duration,
    next_idle: Duration,
}

impl ManagedRuntimePollBackoffV1 {
    pub fn new(
        minimum: Duration,
        maximum: Duration,
    ) -> Result<Self, ManagedRuntimePollBackoffErrorV1> {
        if minimum.is_zero() || maximum < minimum {
            return Err(ManagedRuntimePollBackoffErrorV1::InvalidBounds);
        }
        Ok(Self {
            minimum,
            maximum,
            next_idle: minimum,
        })
    }

    /// Returns the delay before the next loop iteration.
    ///
    /// Progress resets the backoff and returns zero so a burst is drained
    /// without an artificial per-item delay. Consecutive idle observations
    /// double up to the caller-selected maximum.
    pub fn observe(&mut self, progressed: bool) -> Duration {
        if progressed {
            self.next_idle = self.minimum;
            return Duration::ZERO;
        }
        let delay = self.next_idle;
        self.next_idle = self.next_idle.saturating_mul(2).min(self.maximum);
        delay
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedRuntimePollBackoffErrorV1 {
    InvalidBounds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_delay_is_bounded_and_progress_resets_it() {
        let mut backoff =
            ManagedRuntimePollBackoffV1::new(Duration::from_millis(25), Duration::from_millis(100))
                .expect("valid bounds");

        assert_eq!(backoff.observe(false), Duration::from_millis(25));
        assert_eq!(backoff.observe(false), Duration::from_millis(50));
        assert_eq!(backoff.observe(false), Duration::from_millis(100));
        assert_eq!(backoff.observe(false), Duration::from_millis(100));
        assert_eq!(backoff.observe(true), Duration::ZERO);
        assert_eq!(backoff.observe(false), Duration::from_millis(25));
    }

    #[test]
    fn rejects_zero_or_inverted_bounds() {
        assert_eq!(
            ManagedRuntimePollBackoffV1::new(Duration::ZERO, Duration::from_millis(1)),
            Err(ManagedRuntimePollBackoffErrorV1::InvalidBounds),
        );
        assert_eq!(
            ManagedRuntimePollBackoffV1::new(Duration::from_millis(100), Duration::from_millis(25),),
            Err(ManagedRuntimePollBackoffErrorV1::InvalidBounds),
        );
    }
}
