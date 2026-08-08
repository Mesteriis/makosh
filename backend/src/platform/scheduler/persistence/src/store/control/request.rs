use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_scheduler_protocol::{ScheduleIdV1, ScheduleRevisionV1};

use crate::SchedulerScheduleUpsertV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlMutationV1 {
    Ensure(Box<SchedulerScheduleUpsertV1>),
    Cancel {
        schedule_id: ScheduleIdV1,
        expected_revision: ScheduleRevisionV1,
        cancelled_at: UtcMillisV1,
    },
}

impl SchedulerScheduleControlMutationV1 {
    #[must_use]
    pub fn schedule_id(&self) -> ScheduleIdV1 {
        match self {
            Self::Ensure(change) => change.spec().schedule_id(),
            Self::Cancel { schedule_id, .. } => *schedule_id,
        }
    }

    #[must_use]
    pub fn schedule_revision(&self) -> ScheduleRevisionV1 {
        match self {
            Self::Ensure(change) => change.spec().revision(),
            Self::Cancel {
                expected_revision, ..
            } => *expected_revision,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerScheduleControlAuthorityV1 {
    source_module_id: String,
    source_owner: String,
    job_owner: String,
    job_name: String,
    job_major: u16,
}

impl SchedulerScheduleControlAuthorityV1 {
    pub fn new(
        source_module_id: String,
        source_owner: String,
        job_owner: String,
        job_name: String,
        job_major: u16,
    ) -> Result<Self, SchedulerScheduleControlApplyErrorV1> {
        (valid_module_id(&source_module_id)
            && valid_owner(&source_owner)
            && source_owner == job_owner
            && valid_job_token(&job_name)
            && job_major > 0)
            .then_some(Self {
                source_module_id,
                source_owner,
                job_owner,
                job_name,
                job_major,
            })
            .ok_or(SchedulerScheduleControlApplyErrorV1::InvalidRequest)
    }

    #[must_use]
    pub fn source_module_id(&self) -> &str {
        &self.source_module_id
    }

    #[must_use]
    pub fn source_owner(&self) -> &str {
        &self.source_owner
    }

    #[must_use]
    pub fn job_owner(&self) -> &str {
        &self.job_owner
    }

    #[must_use]
    pub fn job_name(&self) -> &str {
        &self.job_name
    }

    #[must_use]
    pub const fn job_major(&self) -> u16 {
        self.job_major
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerScheduleControlRequestV1 {
    command: OutboxRecordV1,
    operation_id: [u8; 16],
    authority: SchedulerScheduleControlAuthorityV1,
    mutation: SchedulerScheduleControlMutationV1,
    received_at: UtcMillisV1,
}

impl SchedulerScheduleControlRequestV1 {
    pub fn new(
        command: OutboxRecordV1,
        operation_id: [u8; 16],
        authority: SchedulerScheduleControlAuthorityV1,
        mutation: SchedulerScheduleControlMutationV1,
        received_at: UtcMillisV1,
    ) -> Result<Self, SchedulerScheduleControlApplyErrorV1> {
        let command_envelope = decode_envelope_v1(command.exact_bytes())
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidRequest)?;
        (operation_id.iter().any(|byte| *byte != 0)
            && matches!(command_envelope.semantics, Some(Semantics::Command(_))))
        .then_some(Self {
            command,
            operation_id,
            authority,
            mutation,
            received_at,
        })
        .ok_or(SchedulerScheduleControlApplyErrorV1::InvalidRequest)
    }

    #[must_use]
    pub fn command(&self) -> &OutboxRecordV1 {
        &self.command
    }

    #[must_use]
    pub const fn operation_id(&self) -> &[u8; 16] {
        &self.operation_id
    }

    #[must_use]
    pub const fn authority(&self) -> &SchedulerScheduleControlAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub const fn mutation(&self) -> &SchedulerScheduleControlMutationV1 {
        &self.mutation
    }

    #[must_use]
    pub const fn received_at(&self) -> UtcMillisV1 {
        self.received_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlRejectionV1 {
    ForeignAuthority,
    UnknownSchedule,
    StaleRevision,
    RevisionConflict,
    ConcurrencyBusy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlDecisionV1 {
    Ensured,
    Cancelled,
    TooLate,
    Rejected(SchedulerScheduleControlRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlApplyOutcomeV1 {
    Applied {
        decision: SchedulerScheduleControlDecisionV1,
        result: OutboxRecordV1,
    },
    Duplicate {
        decision: SchedulerScheduleControlDecisionV1,
        result: OutboxRecordV1,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlApplyErrorV1 {
    InvalidRequest,
    HashConflict,
    InvalidResult,
    CorruptState,
    Unavailable,
}

fn valid_module_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_job_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
