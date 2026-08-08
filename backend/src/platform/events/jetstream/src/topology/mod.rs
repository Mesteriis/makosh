//! Bounded JetStream topology declarations and budgets.

use std::time::Duration;

mod reconciliation;

pub use reconciliation::{EventHubTopologyPlanV1, EventHubTopologyPlanViolationV1};

const MAX_STREAM_BYTES: i64 = 1_073_741_824;
const MAX_RETENTION: Duration = Duration::from_secs(90 * 24 * 60 * 60);
const MAX_ACK_PENDING: i64 = 4_096;
const MAX_DELIVER: i64 = 32;
const MAX_ACK_WAIT: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamKindV1 {
    Command,
    Event,
    Observation,
    Result,
    Ack,
    Dead,
}

impl StreamKindV1 {
    #[must_use]
    pub fn from_subject_token(value: &str) -> Option<Self> {
        match value {
            "command" => Some(Self::Command),
            "event" => Some(Self::Event),
            "observation" => Some(Self::Observation),
            "result" => Some(Self::Result),
            "ack" => Some(Self::Ack),
            "dead" => Some(Self::Dead),
            _ => None,
        }
    }

    #[must_use]
    pub const fn subject_token(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Event => "event",
            Self::Observation => "observation",
            Self::Result => "result",
            Self::Ack => "ack",
            Self::Dead => "dead",
        }
    }

    #[must_use]
    pub const fn stream_name(self) -> &'static str {
        match self {
            Self::Command => "MAKOSH_COMMAND_V1",
            Self::Event => "MAKOSH_EVENT_V1",
            Self::Observation => "MAKOSH_OBSERVATION_V1",
            Self::Result => "MAKOSH_RESULT_V1",
            Self::Ack => "MAKOSH_ACK_V1",
            Self::Dead => "MAKOSH_DEAD_V1",
        }
    }

    #[must_use]
    pub const fn stream_subject(self) -> &'static str {
        match self {
            Self::Command => "makosh.command.v1.>",
            Self::Event => "makosh.event.v1.>",
            Self::Observation => "makosh.observation.v1.>",
            Self::Result => "makosh.result.v1.>",
            Self::Ack => "makosh.ack.v1.>",
            Self::Dead => "makosh.dead.v1.>",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBudgetV1 {
    max_bytes: i64,
    max_age: Duration,
    replicas: usize,
}

impl StreamBudgetV1 {
    pub fn new(max_bytes: i64, max_age: Duration, replicas: usize) -> Result<Self, String> {
        (max_bytes > 0
            && max_bytes <= MAX_STREAM_BYTES
            && !max_age.is_zero()
            && max_age <= MAX_RETENTION
            && replicas == 1)
            .then_some(Self {
                max_bytes,
                max_age,
                replicas,
            })
            .ok_or_else(|| "JetStream stream budget is invalid".to_owned())
    }

    #[must_use]
    pub const fn max_bytes(self) -> i64 {
        self.max_bytes
    }

    #[must_use]
    pub const fn max_age(self) -> Duration {
        self.max_age
    }

    #[must_use]
    pub const fn replicas(self) -> usize {
        self.replicas
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumerBudgetV1 {
    max_ack_pending: i64,
    max_deliver: i64,
    ack_wait: Duration,
}

impl ConsumerBudgetV1 {
    pub fn new(max_ack_pending: i64, max_deliver: i64, ack_wait: Duration) -> Result<Self, String> {
        (max_ack_pending > 0
            && max_ack_pending <= MAX_ACK_PENDING
            && max_deliver > 0
            && max_deliver <= MAX_DELIVER
            && !ack_wait.is_zero()
            && ack_wait <= MAX_ACK_WAIT)
            .then_some(Self {
                max_ack_pending,
                max_deliver,
                ack_wait,
            })
            .ok_or_else(|| "JetStream consumer budget is invalid".to_owned())
    }

    #[must_use]
    pub const fn max_ack_pending(self) -> i64 {
        self.max_ack_pending
    }

    #[must_use]
    pub const fn max_deliver(self) -> i64 {
        self.max_deliver
    }

    #[must_use]
    pub const fn ack_wait(self) -> Duration {
        self.ack_wait
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamSpecV1 {
    kind: StreamKindV1,
    budget: StreamBudgetV1,
}

impl StreamSpecV1 {
    #[must_use]
    pub const fn new(kind: StreamKindV1, budget: StreamBudgetV1) -> Self {
        Self { kind, budget }
    }

    #[must_use]
    pub const fn kind(self) -> StreamKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn budget(self) -> StreamBudgetV1 {
        self.budget
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerSpecV1 {
    stream_kind: StreamKindV1,
    durable_name: String,
    filter_subject: String,
    budget: ConsumerBudgetV1,
}

impl ConsumerSpecV1 {
    pub fn new(
        stream_kind: StreamKindV1,
        durable_name: impl Into<String>,
        filter_subject: impl Into<String>,
        budget: ConsumerBudgetV1,
    ) -> Result<Self, String> {
        let durable_name = durable_name.into();
        let filter_subject = filter_subject.into();
        let prefix = format!("makosh.{}.v1.", stream_kind.subject_token());
        (valid_durable_name(&durable_name)
            && filter_subject.starts_with(&prefix)
            && !filter_subject.contains('>')
            && !filter_subject.contains('*'))
        .then_some(Self {
            stream_kind,
            durable_name,
            filter_subject,
            budget,
        })
        .ok_or_else(|| "JetStream consumer specification is invalid".to_owned())
    }

    #[must_use]
    pub const fn stream_kind(&self) -> StreamKindV1 {
        self.stream_kind
    }

    #[must_use]
    pub fn durable_name(&self) -> &str {
        &self.durable_name
    }

    #[must_use]
    pub fn filter_subject(&self) -> &str {
        &self.filter_subject
    }

    #[must_use]
    pub const fn budget(&self) -> ConsumerBudgetV1 {
        self.budget
    }
}

fn valid_durable_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
