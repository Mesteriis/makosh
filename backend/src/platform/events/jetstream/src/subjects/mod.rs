//! Canonical non-wildcard durable subject grammar.

use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_events_protocol::v1::durable_envelope_v1::Semantics;

use crate::topology::StreamKindV1;

const MAX_SUBJECT_TOKEN_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectError {
    InvalidOwner,
    InvalidContract,
    InvalidMajor,
    EnvelopeHasNoKind,
}

/// Exact subject of one declared durable contract major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableSubjectV1 {
    kind: StreamKindV1,
    owner: String,
    contract: String,
    contract_major: u32,
}

impl DurableSubjectV1 {
    pub fn new(
        kind: StreamKindV1,
        owner: impl Into<String>,
        contract: impl Into<String>,
        contract_major: u32,
    ) -> Result<Self, SubjectError> {
        let owner = owner.into();
        let contract = contract.into();
        if !valid_token(&owner) {
            return Err(SubjectError::InvalidOwner);
        }
        if !valid_token(&contract) {
            return Err(SubjectError::InvalidContract);
        }
        if contract_major == 0 {
            return Err(SubjectError::InvalidMajor);
        }
        Ok(Self {
            kind,
            owner,
            contract,
            contract_major,
        })
    }

    pub fn from_envelope(envelope: &DurableEnvelopeV1) -> Result<Self, SubjectError> {
        let contract = envelope
            .contract
            .as_ref()
            .ok_or(SubjectError::InvalidContract)?;
        Self::new(
            stream_kind(envelope.semantics.as_ref())?,
            &contract.owner,
            &contract.name,
            contract.major,
        )
    }

    pub fn parse(value: &str) -> Result<Self, SubjectError> {
        let mut parts = value.split('.');
        let (
            Some("makosh"),
            Some(kind),
            Some("v1"),
            Some(owner),
            Some(contract),
            Some(major),
            None,
        ) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        )
        else {
            return Err(SubjectError::InvalidContract);
        };
        let kind = StreamKindV1::from_subject_token(kind).ok_or(SubjectError::InvalidContract)?;
        let major = major
            .strip_prefix('v')
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or(SubjectError::InvalidMajor)?;
        Self::new(kind, owner, contract, major)
    }

    #[must_use]
    pub fn kind(&self) -> StreamKindV1 {
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
    pub const fn contract_major(&self) -> u32 {
        self.contract_major
    }

    #[must_use]
    pub fn as_str(&self) -> String {
        format!(
            "makosh.{}.v1.{}.{}.v{}",
            self.kind.subject_token(),
            self.owner,
            self.contract,
            self.contract_major,
        )
    }
}

fn stream_kind(semantics: Option<&Semantics>) -> Result<StreamKindV1, SubjectError> {
    match semantics {
        Some(Semantics::Command(_)) => Ok(StreamKindV1::Command),
        Some(Semantics::Event(_)) => Ok(StreamKindV1::Event),
        Some(Semantics::Observation(_)) => Ok(StreamKindV1::Observation),
        Some(Semantics::Result(_)) => Ok(StreamKindV1::Result),
        Some(Semantics::Ack(_)) => Ok(StreamKindV1::Ack),
        None => Err(SubjectError::EnvelopeHasNoKind),
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SUBJECT_TOKEN_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}
