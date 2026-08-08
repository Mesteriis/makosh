use makosh_clock_protocol::UtcMillisV1;

use crate::contracts::{job::JobKindV1, run::JobRunIdV1};
use crate::v1::{JobKindV1 as WireJobKindV1, JobLeaseV1, OwnerJobCommandV1, OwnerJobTriggerKindV1};
use crate::validation::validate_owner_job_command_v1;

const MAX_SCOPE_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueOwnerJobScopeV1(String);

impl OpaqueOwnerJobScopeV1 {
    pub fn new(value: String) -> Result<Self, OwnerJobCommandBuildErrorV1> {
        (!value.is_empty() && value.len() <= MAX_SCOPE_BYTES && value.is_ascii())
            .then_some(Self(value))
            .ok_or(OwnerJobCommandBuildErrorV1::InvalidScope)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerJobCommandBuildErrorV1 {
    InvalidScope,
    InvalidTrigger,
    InvalidLease,
    InvalidCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerJobLeaseV1 {
    run_id: JobRunIdV1,
    epoch: u64,
    expires_at: UtcMillisV1,
}

impl OwnerJobLeaseV1 {
    pub fn new(
        run_id: JobRunIdV1,
        epoch: u64,
        expires_at: UtcMillisV1,
    ) -> Result<Self, OwnerJobCommandBuildErrorV1> {
        (epoch > 0 && expires_at.value() > 0)
            .then_some(Self {
                run_id,
                epoch,
                expires_at,
            })
            .ok_or(OwnerJobCommandBuildErrorV1::InvalidLease)
    }
}

pub fn build_owner_job_command_v1(
    job_kind: &JobKindV1,
    scope: &OpaqueOwnerJobScopeV1,
    trigger: OwnerJobTriggerKindV1,
    accepted_at: UtcMillisV1,
    lease: OwnerJobLeaseV1,
) -> Result<OwnerJobCommandV1, OwnerJobCommandBuildErrorV1> {
    if trigger == OwnerJobTriggerKindV1::Unspecified {
        return Err(OwnerJobCommandBuildErrorV1::InvalidTrigger);
    }
    if lease.expires_at.value() <= accepted_at.value() {
        return Err(OwnerJobCommandBuildErrorV1::InvalidLease);
    }
    let run_id = lease.run_id.bytes().to_vec();
    let command = OwnerJobCommandV1 {
        job_run_id: run_id.clone(),
        job_kind: Some(WireJobKindV1 {
            owner: job_kind.owner().to_owned(),
            name: job_kind.name().to_owned(),
            major: u32::from(job_kind.major()),
        }),
        scope_id: scope.value().to_owned(),
        trigger_kind: trigger as i32,
        accepted_at_unix_millis: accepted_at.value(),
        lease: Some(JobLeaseV1 {
            run_id,
            epoch: lease.epoch,
            expires_at_unix_millis: lease.expires_at.value(),
        }),
    };
    validate_owner_job_command_v1(&command)
        .map(|_| command)
        .map_err(|_| OwnerJobCommandBuildErrorV1::InvalidCommand)
}
