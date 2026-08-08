use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{ScheduleRunLeaseV1, ScheduleSpecV1};

/// Persistable control state for one schedule; it contains no job payload or owner data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleEntryV1 {
    spec: ScheduleSpecV1,
    next_due_at: UtcMillisV1,
    lease: Option<ScheduleRunLeaseV1>,
}

impl ScheduleEntryV1 {
    #[must_use]
    pub const fn new(spec: ScheduleSpecV1, next_due_at: UtcMillisV1) -> Self {
        Self {
            spec,
            next_due_at,
            lease: None,
        }
    }

    #[must_use]
    pub fn spec(&self) -> &ScheduleSpecV1 {
        &self.spec
    }

    #[must_use]
    pub const fn next_due_at(&self) -> UtcMillisV1 {
        self.next_due_at
    }

    #[must_use]
    pub fn lease_state(&self, now: UtcMillisV1) -> ScheduleLeaseStateV1 {
        match &self.lease {
            Some(lease) if lease.expires_at() > now => ScheduleLeaseStateV1::Active(lease.clone()),
            Some(_) => ScheduleLeaseStateV1::Expired,
            None => ScheduleLeaseStateV1::Vacant,
        }
    }

    pub(super) fn replace_spec(&mut self, spec: ScheduleSpecV1, next_due_at: UtcMillisV1) {
        self.spec = spec;
        self.next_due_at = next_due_at;
    }

    pub(super) fn claim(&mut self, lease: ScheduleRunLeaseV1) {
        self.lease = Some(lease);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleLeaseStateV1 {
    Vacant,
    Active(ScheduleRunLeaseV1),
    Expired,
}
