//! Bounded schedule policy and revision contracts; no owner job code lives here.

mod encoding;

use makosh_clock_protocol::{TimeZoneContextV1, UtcMillisV1};

use crate::contracts::job::JobContractBindingV1;

const MIN_INTERVAL_MILLIS: u64 = 1_000;
const MAX_INTERVAL_MILLIS: u64 = 31_536_000_000;
const MAX_SCOPE_BYTES: usize = 256;
const MAX_CONCURRENCY_KEY_BYTES: usize = 256;
const MAX_CRON_BYTES: usize = 128;
const MAX_CONCURRENCY: u16 = 128;
const MAX_RETRY_ATTEMPTS: u16 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScheduleIdV1([u8; 16]);

impl ScheduleIdV1 {
    pub fn new(value: [u8; 16]) -> Result<Self, ScheduleErrorV1> {
        value
            .iter()
            .any(|byte| *byte != 0)
            .then_some(Self(value))
            .ok_or(ScheduleErrorV1::InvalidScheduleId)
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScheduleRevisionV1(u64);

impl ScheduleRevisionV1 {
    pub fn new(value: u64) -> Result<Self, ScheduleErrorV1> {
        (value > 0)
            .then_some(Self(value))
            .ok_or(ScheduleErrorV1::InvalidRevision)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueScheduleScopeV1(String);

impl OpaqueScheduleScopeV1 {
    pub fn new(value: String) -> Result<Self, ScheduleErrorV1> {
        (!value.is_empty() && value.len() <= MAX_SCOPE_BYTES && value.is_ascii())
            .then_some(Self(value))
            .ok_or(ScheduleErrorV1::InvalidScope)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Opaque coordination boundary for runs that must share an overlap limit.
///
/// A mailbox synchronizer, for example, uses one key per mailbox. The key is
/// technical metadata and must not contain an address, a secret, or job data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcurrencyKeyV1(String);

impl ConcurrencyKeyV1 {
    pub fn new(value: String) -> Result<Self, ScheduleErrorV1> {
        valid_concurrency_key(&value)
            .then_some(Self(value))
            .ok_or(ScheduleErrorV1::InvalidConcurrencyKey)
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleTriggerV1 {
    At {
        due_at: UtcMillisV1,
    },
    FixedInterval {
        interval_millis: u64,
    },
    FixedDelay {
        delay_millis: u64,
    },
    Cron {
        expression: String,
        timezone: TimeZoneContextV1,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlapPolicyV1 {
    Forbid,
    Queue { max_pending_runs: u16 },
    CoalesceLatest,
    AllowBounded { max_parallelism: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MisfirePolicyV1 {
    Skip,
    FireOnce,
    CatchUpBounded { max_runs: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicyV1 {
    max_attempts: u16,
    base_backoff_millis: u64,
}

impl RetryPolicyV1 {
    pub fn new(max_attempts: u16, base_backoff_millis: u64) -> Result<Self, ScheduleErrorV1> {
        (max_attempts > 0
            && max_attempts <= MAX_RETRY_ATTEMPTS
            && valid_interval(base_backoff_millis))
        .then_some(Self {
            max_attempts,
            base_backoff_millis,
        })
        .ok_or(ScheduleErrorV1::InvalidRetryPolicy)
    }

    #[must_use]
    pub const fn max_attempts(self) -> u16 {
        self.max_attempts
    }

    #[must_use]
    pub const fn base_backoff_millis(self) -> u64 {
        self.base_backoff_millis
    }

    /// Returns the bounded backoff after a failed one-based attempt.
    #[must_use]
    pub fn delay_after_failure(self, failed_attempt: u16) -> Option<u64> {
        if failed_attempt == 0 || failed_attempt >= self.max_attempts {
            return None;
        }
        let factor = 1_u64
            .checked_shl(u32::from(failed_attempt - 1))
            .unwrap_or(u64::MAX);
        Some(
            self.base_backoff_millis
                .saturating_mul(factor)
                .min(MAX_INTERVAL_MILLIS),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePolicyV1 {
    trigger: ScheduleTriggerV1,
    overlap: OverlapPolicyV1,
    misfire: MisfirePolicyV1,
    retry: RetryPolicyV1,
    deadline_millis: u64,
    jitter_millis: u64,
}

impl SchedulePolicyV1 {
    pub fn new(
        trigger: ScheduleTriggerV1,
        overlap: OverlapPolicyV1,
        misfire: MisfirePolicyV1,
        retry: RetryPolicyV1,
        deadline_millis: u64,
        jitter_millis: u64,
    ) -> Result<Self, ScheduleErrorV1> {
        validate_trigger(&trigger)?;
        validate_overlap(overlap)?;
        validate_misfire(misfire)?;
        if !valid_interval(deadline_millis) || jitter_millis > deadline_millis {
            return Err(ScheduleErrorV1::InvalidTimingPolicy);
        }
        Ok(Self {
            trigger,
            overlap,
            misfire,
            retry,
            deadline_millis,
            jitter_millis,
        })
    }

    #[must_use]
    pub fn trigger(&self) -> &ScheduleTriggerV1 {
        &self.trigger
    }

    #[must_use]
    pub const fn overlap(&self) -> OverlapPolicyV1 {
        self.overlap
    }

    #[must_use]
    pub const fn max_parallelism(&self) -> u16 {
        match self.overlap {
            OverlapPolicyV1::AllowBounded { max_parallelism } => max_parallelism,
            OverlapPolicyV1::Forbid
            | OverlapPolicyV1::Queue { .. }
            | OverlapPolicyV1::CoalesceLatest => 1,
        }
    }

    #[must_use]
    pub const fn misfire(&self) -> MisfirePolicyV1 {
        self.misfire
    }

    #[must_use]
    pub const fn retry(&self) -> RetryPolicyV1 {
        self.retry
    }

    #[must_use]
    pub const fn deadline_millis(&self) -> u64 {
        self.deadline_millis
    }

    #[must_use]
    pub const fn jitter_millis(&self) -> u64 {
        self.jitter_millis
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encoding::encode(self)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, ScheduleCodecErrorV1> {
        encoding::decode(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleSpecV1 {
    schedule_id: ScheduleIdV1,
    revision: ScheduleRevisionV1,
    binding: JobContractBindingV1,
    scope: OpaqueScheduleScopeV1,
    concurrency_key: ConcurrencyKeyV1,
    enabled: bool,
    policy: SchedulePolicyV1,
}

impl ScheduleSpecV1 {
    #[must_use]
    pub const fn new(
        schedule_id: ScheduleIdV1,
        revision: ScheduleRevisionV1,
        binding: JobContractBindingV1,
        scope: OpaqueScheduleScopeV1,
        concurrency_key: ConcurrencyKeyV1,
        enabled: bool,
        policy: SchedulePolicyV1,
    ) -> Self {
        Self {
            schedule_id,
            revision,
            binding,
            scope,
            concurrency_key,
            enabled,
            policy,
        }
    }

    #[must_use]
    pub const fn schedule_id(&self) -> ScheduleIdV1 {
        self.schedule_id
    }

    #[must_use]
    pub const fn revision(&self) -> ScheduleRevisionV1 {
        self.revision
    }

    #[must_use]
    pub fn binding(&self) -> &JobContractBindingV1 {
        &self.binding
    }

    #[must_use]
    pub fn scope(&self) -> &OpaqueScheduleScopeV1 {
        &self.scope
    }

    #[must_use]
    pub fn concurrency_key(&self) -> &ConcurrencyKeyV1 {
        &self.concurrency_key
    }

    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn policy(&self) -> &SchedulePolicyV1 {
        &self.policy
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleErrorV1 {
    InvalidScheduleId,
    InvalidRevision,
    InvalidScope,
    InvalidConcurrencyKey,
    InvalidTrigger,
    InvalidOverlapPolicy,
    InvalidMisfirePolicy,
    InvalidRetryPolicy,
    InvalidTimingPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleCodecErrorV1 {
    InvalidEncoding,
}

fn validate_trigger(trigger: &ScheduleTriggerV1) -> Result<(), ScheduleErrorV1> {
    match trigger {
        ScheduleTriggerV1::At { .. } => Ok(()),
        ScheduleTriggerV1::FixedInterval { interval_millis }
        | ScheduleTriggerV1::FixedDelay {
            delay_millis: interval_millis,
        } if valid_interval(*interval_millis) => Ok(()),
        ScheduleTriggerV1::Cron { expression, .. } if valid_cron(expression) => Ok(()),
        _ => Err(ScheduleErrorV1::InvalidTrigger),
    }
}

fn validate_overlap(policy: OverlapPolicyV1) -> Result<(), ScheduleErrorV1> {
    match policy {
        OverlapPolicyV1::Queue { max_pending_runs }
            if max_pending_runs == 0 || max_pending_runs > MAX_CONCURRENCY =>
        {
            Err(ScheduleErrorV1::InvalidOverlapPolicy)
        }
        OverlapPolicyV1::AllowBounded { max_parallelism }
            if max_parallelism == 0 || max_parallelism > MAX_CONCURRENCY =>
        {
            Err(ScheduleErrorV1::InvalidOverlapPolicy)
        }
        _ => Ok(()),
    }
}

fn validate_misfire(policy: MisfirePolicyV1) -> Result<(), ScheduleErrorV1> {
    match policy {
        MisfirePolicyV1::CatchUpBounded { max_runs }
            if max_runs == 0 || max_runs > MAX_CONCURRENCY =>
        {
            Err(ScheduleErrorV1::InvalidMisfirePolicy)
        }
        _ => Ok(()),
    }
}

fn valid_interval(value: u64) -> bool {
    (MIN_INTERVAL_MILLIS..=MAX_INTERVAL_MILLIS).contains(&value)
}

fn valid_concurrency_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CONCURRENCY_KEY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':'))
}

fn valid_cron(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_CRON_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'*' | b'/' | b',' | b'-' | b'?')
        })
}
