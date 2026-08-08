//! Run identity and lease fencing for one scheduled fire point.

use makosh_clock_protocol::UtcMillisV1;

use crate::contracts::schedule::{ScheduleIdV1, ScheduleRevisionV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JobRunIdV1([u8; 16]);

impl JobRunIdV1 {
    pub fn new(value: [u8; 16]) -> Result<Self, JobRunErrorV1> {
        value
            .iter()
            .any(|byte| *byte != 0)
            .then_some(Self(value))
            .ok_or(JobRunErrorV1::InvalidRunId)
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleRunLeaseV1 {
    run_id: JobRunIdV1,
    schedule_id: ScheduleIdV1,
    schedule_revision: ScheduleRevisionV1,
    epoch: u64,
    expires_at: UtcMillisV1,
}

impl ScheduleRunLeaseV1 {
    pub fn new(
        run_id: JobRunIdV1,
        schedule_id: ScheduleIdV1,
        schedule_revision: ScheduleRevisionV1,
        epoch: u64,
        expires_at: UtcMillisV1,
    ) -> Result<Self, JobRunErrorV1> {
        (epoch > 0)
            .then_some(Self {
                run_id,
                schedule_id,
                schedule_revision,
                epoch,
                expires_at,
            })
            .ok_or(JobRunErrorV1::InvalidLease)
    }

    #[must_use]
    pub const fn run_id(&self) -> JobRunIdV1 {
        self.run_id
    }

    #[must_use]
    pub const fn schedule_id(&self) -> ScheduleIdV1 {
        self.schedule_id
    }

    #[must_use]
    pub const fn schedule_revision(&self) -> ScheduleRevisionV1 {
        self.schedule_revision
    }

    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    #[must_use]
    pub const fn expires_at(&self) -> UtcMillisV1 {
        self.expires_at
    }

    #[must_use]
    pub fn accepts_completion(&self, epoch: u64, now: UtcMillisV1) -> bool {
        epoch == self.epoch && now < self.expires_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobRunErrorV1 {
    InvalidRunId,
    InvalidLease,
}
