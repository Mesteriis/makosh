use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobRunIdV1, ScheduleIdV1, SchedulePolicyV1, ScheduleRevisionV1,
    ScheduleRunLeaseV1, ScheduleTriggerV1,
};

use super::SchedulerRunClaimErrorV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerRunClaimV1 {
    lease: ScheduleRunLeaseV1,
    scheduled_for: UtcMillisV1,
    claimed_at: UtcMillisV1,
    next_due_at: UtcMillisV1,
    concurrency_key: ConcurrencyKeyV1,
    policy: SchedulePolicyV1,
    max_parallelism: u16,
    dispatch_message_id: [u8; 16],
    fire_key: [u8; 32],
}

impl SchedulerRunClaimV1 {
    #[allow(clippy::too_many_arguments)] // The persisted claim is validated atomically from its exact durable fields.
    pub fn new(
        lease: ScheduleRunLeaseV1,
        scheduled_for: UtcMillisV1,
        claimed_at: UtcMillisV1,
        next_due_at: UtcMillisV1,
        concurrency_key: ConcurrencyKeyV1,
        policy: &SchedulePolicyV1,
        dispatch_message_id: [u8; 16],
        fire_key: [u8; 32],
    ) -> Result<Self, SchedulerRunClaimRequestErrorV1> {
        (valid_initial_lease(&lease, claimed_at, policy)
            && scheduled_for <= claimed_at
            && scheduled_for <= next_due_at
            && dispatch_message_id.iter().any(|byte| *byte != 0)
            && fire_key.iter().any(|byte| *byte != 0))
        .then_some(Self {
            lease,
            scheduled_for,
            claimed_at,
            next_due_at,
            concurrency_key,
            policy: policy.clone(),
            max_parallelism: policy.max_parallelism(),
            dispatch_message_id,
            fire_key,
        })
        .ok_or(SchedulerRunClaimRequestErrorV1::Invalid)
    }

    #[must_use]
    pub const fn run_id(&self) -> JobRunIdV1 {
        self.lease.run_id()
    }

    #[must_use]
    pub const fn schedule_id(&self) -> ScheduleIdV1 {
        self.lease.schedule_id()
    }

    #[must_use]
    pub const fn schedule_revision(&self) -> ScheduleRevisionV1 {
        self.lease.schedule_revision()
    }

    #[must_use]
    pub const fn scheduled_for(&self) -> UtcMillisV1 {
        self.scheduled_for
    }

    #[must_use]
    pub const fn claimed_at(&self) -> UtcMillisV1 {
        self.claimed_at
    }

    #[must_use]
    pub const fn next_due_at(&self) -> UtcMillisV1 {
        self.next_due_at
    }

    #[must_use]
    pub fn concurrency_key(&self) -> &ConcurrencyKeyV1 {
        &self.concurrency_key
    }

    #[must_use]
    pub fn policy(&self) -> &SchedulePolicyV1 {
        &self.policy
    }

    #[must_use]
    pub fn advances_due_on_claim(&self) -> bool {
        !matches!(self.policy.trigger(), ScheduleTriggerV1::FixedDelay { .. })
    }

    pub(crate) fn validate_renewal(
        &self,
        renewed_at: UtcMillisV1,
        renewed_until: UtcMillisV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let deadline = self
            .claimed_at
            .value()
            .checked_add(
                i64::try_from(self.policy.deadline_millis())
                    .map_err(|_| SchedulerRunClaimErrorV1::Denied)?,
            )
            .ok_or(SchedulerRunClaimErrorV1::Denied)?;
        (renewed_until > renewed_at && renewed_until.value() <= deadline)
            .then_some(())
            .ok_or(SchedulerRunClaimErrorV1::Denied)
    }

    #[must_use]
    pub const fn max_parallelism(&self) -> u16 {
        self.max_parallelism
    }

    #[must_use]
    pub const fn lease_epoch(&self) -> u64 {
        self.lease.epoch()
    }

    #[must_use]
    pub const fn lease_expires_at(&self) -> UtcMillisV1 {
        self.lease.expires_at()
    }

    #[must_use]
    pub const fn dispatch_message_id(&self) -> [u8; 16] {
        self.dispatch_message_id
    }

    #[must_use]
    pub const fn fire_key(&self) -> [u8; 32] {
        self.fire_key
    }
}

fn valid_initial_lease(
    lease: &ScheduleRunLeaseV1,
    claimed_at: UtcMillisV1,
    policy: &SchedulePolicyV1,
) -> bool {
    let Some(deadline) = claimed_at
        .value()
        .checked_add(i64::try_from(policy.deadline_millis()).unwrap_or(i64::MAX))
    else {
        return false;
    };
    lease.expires_at() > claimed_at && lease.expires_at().value() <= deadline
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunClaimRequestErrorV1 {
    Invalid,
}
