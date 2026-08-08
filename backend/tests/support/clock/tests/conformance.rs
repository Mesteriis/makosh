use makosh_clock_protocol::{
    ClockDiscontinuityV1, ClockPolicyV1, TimeZoneContextErrorV1, TimeZoneContextV1,
};
use makosh_clock_runtime::{DeterministicClockV1, SystemClockV1};

#[test]
fn fake_clock_advances_wall_and_monotonic_time_together() {
    let mut clock = DeterministicClockV1::new(1_000);
    clock.advance(250);
    let reading = clock.read();
    assert_eq!(reading.wall_utc().value(), 1_250);
    assert_eq!(reading.monotonic().value(), 250);
    assert_eq!(reading.discontinuity(), ClockDiscontinuityV1::Stable);
}

#[test]
fn fake_clock_reports_wall_jumps_without_rewinding_monotonic_time() {
    let mut clock = DeterministicClockV1::new(1_000);
    clock.advance(100);
    clock.jump_wall(-500);
    let reading = clock.read();
    assert_eq!(reading.wall_utc().value(), 600);
    assert_eq!(reading.monotonic().value(), 100);
    assert_eq!(
        reading.discontinuity(),
        ClockDiscontinuityV1::WallClockJumpBackward
    );
}

#[test]
fn fake_clock_reports_resume_as_a_distinct_discontinuity() {
    let mut clock = DeterministicClockV1::new(1_000);
    clock.resume_after(30_000);
    assert_eq!(
        clock.read().discontinuity(),
        ClockDiscontinuityV1::SuspendOrMonotonicGap
    );
}

#[test]
fn policy_rejects_ambiguous_thresholds() {
    assert!(ClockPolicyV1::new(0, 1).is_err());
    assert!(ClockPolicyV1::new(10, 9).is_err());
}

#[test]
fn timezone_context_requires_explicit_iana_identifier_and_offset() {
    assert!(TimeZoneContextV1::new("Europe/Madrid".into(), 7_200).is_ok());
    assert_eq!(
        TimeZoneContextV1::new("UTC".into(), 0),
        Err(TimeZoneContextErrorV1::InvalidIanaName)
    );
    assert_eq!(
        TimeZoneContextV1::new("Europe/Madrid".into(), 90_000),
        Err(TimeZoneContextErrorV1::OffsetOutOfRange)
    );
}

#[test]
fn system_clock_returns_utc_and_monotonic_reading() {
    let clock = SystemClockV1::new(ClockPolicyV1::production_default());
    let reading = clock.read().expect("system clock is after Unix epoch");
    assert!(reading.wall_utc().value() > 0);
    assert!(reading.monotonic().value() < 1_000);
}
