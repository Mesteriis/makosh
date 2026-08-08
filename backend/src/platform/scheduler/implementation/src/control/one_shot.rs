use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobContractBindingV1, MisfirePolicyV1, OpaqueScheduleScopeV1,
    OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1, SchedulePolicyV1, ScheduleRevisionV1,
    ScheduleSpecV1, ScheduleTriggerV1, v1::EnsureOneShotScheduleV1,
};

/// Exact catalog binding already fenced to the calling runtime by the adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerApprovedJobV1 {
    source_owner: String,
    binding: JobContractBindingV1,
}

impl SchedulerApprovedJobV1 {
    pub fn new(
        source_owner: String,
        binding: JobContractBindingV1,
    ) -> Result<Self, SchedulerOneShotScheduleErrorV1> {
        (source_owner == binding.job_kind().owner())
            .then_some(Self {
                source_owner,
                binding,
            })
            .ok_or(SchedulerOneShotScheduleErrorV1::ForeignJobKind)
    }

    #[must_use]
    pub fn source_owner(&self) -> &str {
        &self.source_owner
    }

    #[must_use]
    pub fn binding(&self) -> &JobContractBindingV1 {
        &self.binding
    }
}

/// Canonical one-shot configuration ready for Scheduler-owned persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerOneShotScheduleV1 {
    spec: ScheduleSpecV1,
    next_due_at: UtcMillisV1,
}

impl SchedulerOneShotScheduleV1 {
    #[must_use]
    pub fn spec(&self) -> &ScheduleSpecV1 {
        &self.spec
    }

    #[must_use]
    pub const fn next_due_at(&self) -> UtcMillisV1 {
        self.next_due_at
    }
}

/// Maps only a previously grant- and catalog-fenced request.
///
/// Runtime generation, grant epoch and catalog lookup stay outside this pure
/// mapper. The mapper still compares every catalog-owned contract field so a
/// transport adapter cannot silently substitute a different job contract.
pub fn map_approved_one_shot_schedule_v1(
    request: &EnsureOneShotScheduleV1,
    approved: &SchedulerApprovedJobV1,
) -> Result<SchedulerOneShotScheduleV1, SchedulerOneShotScheduleErrorV1> {
    let requested_kind = request
        .job_kind
        .as_ref()
        .ok_or(SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let approved_kind = approved.binding().job_kind();
    if requested_kind.owner != approved.source_owner()
        || requested_kind.owner != approved_kind.owner()
        || requested_kind.name != approved_kind.name()
        || u32::from(approved_kind.major()) != requested_kind.major
    {
        return Err(SchedulerOneShotScheduleErrorV1::ForeignJobKind);
    }
    if request.job_contract_revision != u64::from(approved.binding().contract_revision())
        || request.job_schema_sha256 != approved.binding().schema_sha256()
    {
        return Err(SchedulerOneShotScheduleErrorV1::ContractMismatch);
    }

    let schedule_id = ScheduleIdV1::new(fixed::<16>(&request.schedule_id)?)
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let revision = ScheduleRevisionV1::new(request.schedule_revision)
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let scope = OpaqueScheduleScopeV1::new(request.scope_id.clone())
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let concurrency_key = ConcurrencyKeyV1::new(request.concurrency_key.clone())
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let max_attempts = u16::try_from(request.max_attempts)
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let retry = RetryPolicyV1::new(max_attempts, request.retry_base_backoff_millis)
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    let due_at = UtcMillisV1::new(request.due_at_unix_millis);
    let policy = SchedulePolicyV1::new(
        ScheduleTriggerV1::At { due_at },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::FireOnce,
        retry,
        request.deadline_millis,
        0,
    )
    .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)?;
    Ok(SchedulerOneShotScheduleV1 {
        spec: ScheduleSpecV1::new(
            schedule_id,
            revision,
            approved.binding().clone(),
            scope,
            concurrency_key,
            true,
            policy,
        ),
        next_due_at: due_at,
    })
}

fn fixed<const N: usize>(value: &[u8]) -> Result<[u8; N], SchedulerOneShotScheduleErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerOneShotScheduleErrorV1::InvalidRequest)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerOneShotScheduleErrorV1 {
    InvalidRequest,
    ForeignJobKind,
    ContractMismatch,
}
