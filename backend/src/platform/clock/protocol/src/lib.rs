//! Public, UTC-only time contracts for Макошь platform components.

mod time;

pub use time::{
    ClockDiscontinuityV1, ClockPolicyErrorV1, ClockPolicyV1, ClockReadingV1, MonotonicMillisV1,
    TimeZoneContextErrorV1, TimeZoneContextV1, UtcMillisV1,
};
