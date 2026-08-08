//! Public, owner-neutral contracts for Scheduler dispatch materialization.

use std::collections::BTreeSet;

use makosh_scheduler::SchedulerDispatchIdentityV1;

use super::SchedulerDueScheduleV1;

/// Fenced Scheduler process identity embedded in newly materialized commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerMaterializationSourceV1 {
    runtime_id: String,
    runtime_instance_id: [u8; 16],
    runtime_generation: u64,
}

impl SchedulerMaterializationSourceV1 {
    pub fn new(
        runtime_id: String,
        runtime_instance_id: [u8; 16],
        runtime_generation: u64,
    ) -> Result<Self, SchedulerMaterializationErrorV1> {
        SchedulerDispatchIdentityV1::new(
            runtime_id.clone(),
            runtime_instance_id,
            runtime_generation,
        )
        .map(|_| Self {
            runtime_id,
            runtime_instance_id,
            runtime_generation,
        })
        .map_err(|_| SchedulerMaterializationErrorV1::InvalidSource)
    }

    pub(super) fn dispatch_identity(
        &self,
    ) -> Result<SchedulerDispatchIdentityV1, SchedulerMaterializationErrorV1> {
        SchedulerDispatchIdentityV1::new(
            self.runtime_id.clone(),
            self.runtime_instance_id,
            self.runtime_generation,
        )
        .map_err(|_| SchedulerMaterializationErrorV1::InvalidSource)
    }
}

/// Kernel-derived command subjects to which this Scheduler runtime is admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerDispatchAdmissionV1 {
    subjects: BTreeSet<String>,
}

impl SchedulerDispatchAdmissionV1 {
    pub fn new(
        subjects: impl IntoIterator<Item = String>,
    ) -> Result<Self, SchedulerMaterializationErrorV1> {
        let subjects = subjects.into_iter().collect::<BTreeSet<_>>();
        (!subjects.is_empty()
            && subjects
                .iter()
                .all(|subject| valid_command_subject(subject)))
        .then_some(Self { subjects })
        .ok_or(SchedulerMaterializationErrorV1::InvalidAdmission)
    }

    pub(super) fn admits(&self, schedule: &SchedulerDueScheduleV1) -> bool {
        self.subjects.contains(&command_subject(schedule))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SchedulerMaterializationOutcomeV1 {
    pub(super) dispatched: u16,
    pub(super) queued: u16,
    pub(super) dropped: u16,
    pub(super) denied: u16,
}

impl SchedulerMaterializationOutcomeV1 {
    #[must_use]
    pub const fn dispatched(self) -> u16 {
        self.dispatched
    }
    #[must_use]
    pub const fn queued(self) -> u16 {
        self.queued
    }
    #[must_use]
    pub const fn dropped(self) -> u16 {
        self.dropped
    }
    #[must_use]
    pub const fn denied(self) -> u16 {
        self.denied
    }

    pub(super) fn add(&mut self, other: Self) {
        self.dispatched = self.dispatched.saturating_add(other.dispatched);
        self.queued = self.queued.saturating_add(other.queued);
        self.dropped = self.dropped.saturating_add(other.dropped);
        self.denied = self.denied.saturating_add(other.denied);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerMaterializationErrorV1 {
    InvalidAdmission,
    InvalidSource,
    InvalidLimit,
    InvalidTime,
    CorruptState,
    Unavailable,
}

fn command_subject(schedule: &SchedulerDueScheduleV1) -> String {
    let job = schedule.spec().binding().job_kind();
    format!(
        "makosh.command.v1.{}.{}.v{}",
        job.owner(),
        job.name(),
        job.major()
    )
}

fn valid_command_subject(subject: &str) -> bool {
    subject.starts_with("makosh.command.v1.")
        && !subject.contains('>')
        && subject.split('.').all(|segment| !segment.is_empty())
}
