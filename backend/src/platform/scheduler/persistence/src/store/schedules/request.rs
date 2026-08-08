use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::ScheduleSpecV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerScheduleUpsertV1 {
    spec: ScheduleSpecV1,
    next_due_at: UtcMillisV1,
    updated_at: UtcMillisV1,
}

impl SchedulerScheduleUpsertV1 {
    #[must_use]
    pub const fn new(
        spec: ScheduleSpecV1,
        next_due_at: UtcMillisV1,
        updated_at: UtcMillisV1,
    ) -> Self {
        Self {
            spec,
            next_due_at,
            updated_at,
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
    pub const fn updated_at(&self) -> UtcMillisV1 {
        self.updated_at
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerDueScheduleV1 {
    spec: ScheduleSpecV1,
    next_due_at: UtcMillisV1,
}

impl SchedulerDueScheduleV1 {
    pub(super) const fn new(spec: ScheduleSpecV1, next_due_at: UtcMillisV1) -> Self {
        Self { spec, next_due_at }
    }

    #[must_use]
    pub fn spec(&self) -> &ScheduleSpecV1 {
        &self.spec
    }

    #[must_use]
    pub const fn next_due_at(&self) -> UtcMillisV1 {
        self.next_due_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleUpsertOutcomeV1 {
    Inserted,
    Updated,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleStoreErrorV1 {
    StaleRevision,
    RevisionConflict,
    ConcurrencyBusy,
    InvalidLimit,
    CorruptState,
    Unavailable,
}
