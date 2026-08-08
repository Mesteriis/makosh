//! Exact durable subjects and stream kinds without a broker dependency.

use makosh_kernel_control_store::ModuleEventEnvelopeKindV1;

const MAX_SUBJECT_TOKEN_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventStreamKindV1 {
    Command,
    Event,
    Observation,
    Result,
    Ack,
}

impl EventStreamKindV1 {
    pub const fn from_envelope_kind(kind: ModuleEventEnvelopeKindV1) -> Self {
        match kind {
            ModuleEventEnvelopeKindV1::Command => Self::Command,
            ModuleEventEnvelopeKindV1::Event => Self::Event,
            ModuleEventEnvelopeKindV1::Observation => Self::Observation,
            ModuleEventEnvelopeKindV1::Result => Self::Result,
            ModuleEventEnvelopeKindV1::Ack => Self::Ack,
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
        }
    }
}

/// One canonical non-wildcard subject declared by an approved contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventSubjectV1 {
    kind: EventStreamKindV1,
    owner: String,
    contract: String,
    major: u32,
}

impl EventSubjectV1 {
    pub fn new(
        kind: EventStreamKindV1,
        owner: impl Into<String>,
        contract: impl Into<String>,
        major: u32,
    ) -> Result<Self, String> {
        let owner = owner.into();
        let contract = contract.into();
        (valid_token(&owner) && valid_token(&contract) && major > 0)
            .then_some(Self {
                kind,
                owner,
                contract,
                major,
            })
            .ok_or_else(|| "Event catalog has an invalid durable subject".to_owned())
    }

    #[must_use]
    pub const fn kind(&self) -> EventStreamKindV1 {
        self.kind
    }

    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    #[must_use]
    pub fn contract(&self) -> &str {
        &self.contract
    }

    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    #[must_use]
    pub fn as_str(&self) -> String {
        format!(
            "makosh.{}.v1.{}.{}.v{}",
            self.kind.subject_token(),
            self.owner,
            self.contract,
            self.major
        )
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SUBJECT_TOKEN_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
