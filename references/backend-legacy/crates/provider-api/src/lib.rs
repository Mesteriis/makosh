use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use makosh_kernel::identifiers::{IdentifierError, validate_slug};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const PROVIDER_ENVELOPE_VERSION: u16 = 1;
pub const PROVIDER_PAYLOAD_VERSION: u16 = 1;

/// An expiring, account-scoped credential capability handed to a provider runtime.
///
/// Only vault/runtime composition constructs a lease. Provider implementations
/// receive it as an opaque secret capability and must not persist it.
#[derive(Deserialize, Eq, PartialEq, Serialize, Zeroize, ZeroizeOnDrop)]
pub struct CredentialLease {
    #[zeroize(skip)]
    pub provider_id: String,
    #[zeroize(skip)]
    pub account_id: String,
    #[zeroize(skip)]
    pub purpose: String,
    #[zeroize(skip)]
    pub epoch: u64,
    #[zeroize(skip)]
    pub issued_at: DateTime<Utc>,
    #[zeroize(skip)]
    pub expires_at: DateTime<Utc>,
    secret: Vec<u8>,
}

impl CredentialLease {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider_id: impl AsRef<str>,
        account_id: impl Into<String>,
        purpose: impl Into<String>,
        epoch: u64,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        secret: impl AsRef<[u8]>,
    ) -> Result<Self, ProviderContractError> {
        let provider_id = provider_id.as_ref().trim();
        validate_slug(provider_id, "provider id")
            .map_err(ProviderContractError::InvalidProviderId)?;
        if expires_at <= issued_at {
            return Err(ProviderContractError::ExpiredCredentialLease);
        }
        let secret = secret.as_ref().to_vec();
        if secret.is_empty() {
            return Err(ProviderContractError::EmptyCredentialLeaseSecret);
        }
        Ok(Self {
            provider_id: provider_id.to_owned(),
            account_id: required_identifier(account_id.into(), "account id")?,
            purpose: required_identifier(purpose.into(), "credential purpose")?,
            epoch,
            issued_at,
            expires_at,
            secret,
        })
    }

    pub fn is_expired_at(&self, at: DateTime<Utc>) -> bool {
        at >= self.expires_at
    }

    pub fn secret(&self) -> &[u8] {
        &self.secret
    }
}

impl fmt::Debug for CredentialLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CredentialLease")
            .field("provider_id", &self.provider_id)
            .field("account_id", &self.account_id)
            .field("purpose", &self.purpose)
            .field("epoch", &self.epoch)
            .field("issued_at", &self.issued_at)
            .field("expires_at", &self.expires_at)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ProviderContractError> {
        let value = value.as_ref().trim();
        validate_slug(value, "provider id").map_err(ProviderContractError::InvalidProviderId)?;
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTopology {
    InProcess,
    SharedConnector,
    PerAccountConnector,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderManifest {
    pub provider_id: ProviderId,
    pub protocol_version: u32,
    pub capabilities: BTreeSet<String>,
    pub supported_topologies: BTreeSet<RuntimeTopology>,
}

impl ProviderManifest {
    pub fn new(
        provider_id: ProviderId,
        protocol_version: u32,
        capabilities: impl IntoIterator<Item = impl Into<String>>,
        supported_topologies: impl IntoIterator<Item = RuntimeTopology>,
    ) -> Result<Self, ProviderContractError> {
        if protocol_version == 0 {
            return Err(ProviderContractError::InvalidProtocolVersion);
        }

        let capabilities = capabilities
            .into_iter()
            .map(Into::into)
            .map(|capability: String| capability.trim().to_owned())
            .collect::<BTreeSet<_>>();
        if capabilities.is_empty() || capabilities.iter().any(|capability| capability.is_empty()) {
            return Err(ProviderContractError::EmptyCapabilities);
        }

        let supported_topologies = supported_topologies.into_iter().collect::<BTreeSet<_>>();
        if supported_topologies.is_empty() {
            return Err(ProviderContractError::EmptyTopologies);
        }

        Ok(Self {
            provider_id,
            protocol_version,
            capabilities,
            supported_topologies,
        })
    }

    pub fn supports(&self, topology: RuntimeTopology) -> bool {
        self.supported_topologies.contains(&topology)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCommandInput {
    pub command_id: String,
    pub idempotency_key: String,
    pub provider_id: ProviderId,
    pub account_id: String,
    pub issued_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub attempt: u32,
    pub lease_epoch: u64,
    pub payload: Value,
    pub envelope_version: u16,
    pub payload_version: u16,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
}

impl ProviderCommandInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command_id: impl Into<String>,
        idempotency_key: impl Into<String>,
        provider_id: ProviderId,
        account_id: impl Into<String>,
        issued_at: DateTime<Utc>,
        deadline: DateTime<Utc>,
        attempt: u32,
        lease_epoch: u64,
        payload: Value,
    ) -> Self {
        Self {
            command_id: command_id.into(),
            idempotency_key: idempotency_key.into(),
            provider_id,
            account_id: account_id.into(),
            issued_at,
            deadline,
            attempt,
            lease_epoch,
            payload,
            envelope_version: PROVIDER_ENVELOPE_VERSION,
            payload_version: PROVIDER_PAYLOAD_VERSION,
            causation_id: None,
            correlation_id: None,
        }
    }

    pub fn with_causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderCommandEnvelope {
    pub envelope_version: u16,
    pub payload_version: u16,
    pub command_id: String,
    pub idempotency_key: String,
    pub provider_id: ProviderId,
    pub account_id: String,
    pub issued_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub attempt: u32,
    pub lease_epoch: u64,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub payload: Value,
}

impl TryFrom<ProviderCommandInput> for ProviderCommandEnvelope {
    type Error = ProviderContractError;

    fn try_from(input: ProviderCommandInput) -> Result<Self, Self::Error> {
        validate_versions(input.envelope_version, input.payload_version)?;
        if input.deadline <= input.issued_at {
            return Err(ProviderContractError::ExpiredDeadline);
        }
        if !input.payload.is_object() {
            return Err(ProviderContractError::NonObjectPayload);
        }

        Ok(Self {
            envelope_version: input.envelope_version,
            payload_version: input.payload_version,
            command_id: required_identifier(input.command_id, "command id")?,
            idempotency_key: required_identifier(input.idempotency_key, "idempotency key")?,
            provider_id: input.provider_id,
            account_id: required_identifier(input.account_id, "account id")?,
            issued_at: input.issued_at,
            deadline: input.deadline,
            attempt: input.attempt,
            lease_epoch: input.lease_epoch,
            causation_id: optional_identifier(input.causation_id, "causation id")?,
            correlation_id: optional_identifier(input.correlation_id, "correlation id")?,
            payload: input.payload,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderObservationInput {
    pub observation_id: String,
    pub provider_id: ProviderId,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
    pub provider_cursor: String,
    pub spool_sequence: Option<u64>,
    pub lease_epoch: u64,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub envelope_version: u16,
    pub payload_version: u16,
    pub payload: Value,
    pub provenance: Value,
}

impl ProviderObservationInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        observation_id: impl Into<String>,
        provider_id: ProviderId,
        account_id: impl Into<String>,
        record_kind: impl Into<String>,
        provider_record_id: impl Into<String>,
        source_fingerprint: impl Into<String>,
        import_batch_id: impl Into<String>,
        observed_at: DateTime<Utc>,
        occurred_at: DateTime<Utc>,
        provider_cursor: impl Into<String>,
        lease_epoch: u64,
        payload: Value,
        provenance: Value,
    ) -> Self {
        Self {
            observation_id: observation_id.into(),
            provider_id,
            account_id: account_id.into(),
            record_kind: record_kind.into(),
            provider_record_id: provider_record_id.into(),
            source_fingerprint: source_fingerprint.into(),
            import_batch_id: import_batch_id.into(),
            observed_at,
            occurred_at,
            provider_cursor: provider_cursor.into(),
            spool_sequence: None,
            lease_epoch,
            causation_id: None,
            correlation_id: None,
            envelope_version: PROVIDER_ENVELOPE_VERSION,
            payload_version: PROVIDER_PAYLOAD_VERSION,
            payload,
            provenance,
        }
    }

    pub fn with_spool_sequence(mut self, spool_sequence: u64) -> Self {
        self.spool_sequence = Some(spool_sequence);
        self
    }

    pub fn with_lease_epoch(mut self, lease_epoch: u64) -> Self {
        self.lease_epoch = lease_epoch;
        self
    }

    pub fn with_causation_id(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderObservationEnvelope {
    pub envelope_version: u16,
    pub payload_version: u16,
    pub observation_id: String,
    pub provider_id: ProviderId,
    pub account_id: String,
    pub record_kind: String,
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
    pub provider_cursor: String,
    pub spool_sequence: Option<u64>,
    pub lease_epoch: u64,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub payload: Value,
    pub provenance: Value,
}

impl TryFrom<ProviderObservationInput> for ProviderObservationEnvelope {
    type Error = ProviderContractError;

    fn try_from(input: ProviderObservationInput) -> Result<Self, Self::Error> {
        validate_versions(input.envelope_version, input.payload_version)?;
        if !input.payload.is_object() {
            return Err(ProviderContractError::NonObjectPayload);
        }
        if !input.provenance.is_object() {
            return Err(ProviderContractError::NonObjectProvenance);
        }

        Ok(Self {
            envelope_version: input.envelope_version,
            payload_version: input.payload_version,
            observation_id: required_identifier(input.observation_id, "observation id")?,
            provider_id: input.provider_id,
            account_id: required_identifier(input.account_id, "account id")?,
            record_kind: required_identifier(input.record_kind, "record kind")?,
            provider_record_id: required_identifier(
                input.provider_record_id,
                "provider record id",
            )?,
            source_fingerprint: required_identifier(
                input.source_fingerprint,
                "source fingerprint",
            )?,
            import_batch_id: required_identifier(input.import_batch_id, "import batch id")?,
            observed_at: input.observed_at,
            occurred_at: input.occurred_at,
            provider_cursor: required_identifier(input.provider_cursor, "provider cursor")?,
            spool_sequence: input.spool_sequence,
            lease_epoch: input.lease_epoch,
            causation_id: optional_identifier(input.causation_id, "causation id")?,
            correlation_id: optional_identifier(input.correlation_id, "correlation id")?,
            payload: input.payload,
            provenance: input.provenance,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCommandDisposition {
    Completed,
    Failed,
    UnknownOutcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderCommandResult {
    pub command_id: String,
    pub provider_id: ProviderId,
    pub account_id: String,
    pub lease_epoch: u64,
    pub completed_at: DateTime<Utc>,
    pub disposition: ProviderCommandDisposition,
    pub payload_version: u16,
    pub payload: Value,
}

impl ProviderCommandResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command_id: impl Into<String>,
        provider_id: ProviderId,
        account_id: impl Into<String>,
        lease_epoch: u64,
        completed_at: DateTime<Utc>,
        disposition: ProviderCommandDisposition,
        payload_version: u16,
        payload: Value,
    ) -> Result<Self, ProviderContractError> {
        if payload_version == 0 {
            return Err(ProviderContractError::InvalidPayloadVersion);
        }
        if !payload.is_object() {
            return Err(ProviderContractError::NonObjectPayload);
        }
        Ok(Self {
            command_id: required_identifier(command_id.into(), "command id")?,
            provider_id,
            account_id: required_identifier(account_id.into(), "account id")?,
            lease_epoch,
            completed_at,
            disposition,
            payload_version,
            payload,
        })
    }
}

pub type ProviderRuntimePortFuture<'a> = Pin<
    Box<dyn Future<Output = Result<ProviderCommandResult, ProviderRuntimePortError>> + Send + 'a>,
>;

pub trait ProviderRuntimePort: Send + Sync {
    fn manifest(&self) -> ProviderManifest;

    fn execute<'a>(
        &'a self,
        command: &'a ProviderCommandEnvelope,
        credential: CredentialLease,
    ) -> ProviderRuntimePortFuture<'a>;
}

#[derive(Debug, Error, Eq, PartialEq)]
#[error("provider runtime rejected {operation}: {code}")]
pub struct ProviderRuntimePortError {
    pub operation: &'static str,
    pub code: String,
    pub retryable: bool,
}

impl ProviderRuntimePortError {
    pub fn new(operation: &'static str, code: impl Into<String>, retryable: bool) -> Self {
        Self {
            operation,
            code: code.into(),
            retryable,
        }
    }
}

fn validate_versions(
    envelope_version: u16,
    payload_version: u16,
) -> Result<(), ProviderContractError> {
    if envelope_version == 0 {
        return Err(ProviderContractError::InvalidEnvelopeVersion);
    }
    if payload_version == 0 {
        return Err(ProviderContractError::InvalidPayloadVersion);
    }
    Ok(())
}

fn required_identifier(value: String, kind: &'static str) -> Result<String, ProviderContractError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ProviderContractError::EmptyField { kind });
    }
    Ok(value.to_owned())
}

fn optional_identifier(
    value: Option<String>,
    kind: &'static str,
) -> Result<Option<String>, ProviderContractError> {
    value
        .map(|value| required_identifier(value, kind))
        .transpose()
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProviderContractError {
    #[error("invalid provider id: {0}")]
    InvalidProviderId(IdentifierError),
    #[error("provider protocol version must be positive")]
    InvalidProtocolVersion,
    #[error("provider manifest must declare at least one capability")]
    EmptyCapabilities,
    #[error("provider manifest must declare at least one supported topology")]
    EmptyTopologies,
    #[error("{kind} must not be empty")]
    EmptyField { kind: &'static str },
    #[error("provider envelope version must be positive")]
    InvalidEnvelopeVersion,
    #[error("provider payload version must be positive")]
    InvalidPayloadVersion,
    #[error("provider command deadline must be after issuance")]
    ExpiredDeadline,
    #[error("provider envelope payload must be a JSON object")]
    NonObjectPayload,
    #[error("provider observation provenance must be a JSON object")]
    NonObjectProvenance,
    #[error("credential lease expiry must be after issuance")]
    ExpiredCredentialLease,
    #[error("credential lease secret must not be empty")]
    EmptyCredentialLeaseSecret,
}

impl ProviderContractError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidProviderId(_) => "invalid_provider_id",
            Self::InvalidProtocolVersion => "invalid_protocol_version",
            Self::EmptyCapabilities => "empty_capabilities",
            Self::EmptyTopologies => "empty_topologies",
            Self::EmptyField { .. } => "empty_field",
            Self::InvalidEnvelopeVersion => "invalid_envelope_version",
            Self::InvalidPayloadVersion => "invalid_payload_version",
            Self::ExpiredDeadline => "expired_deadline",
            Self::NonObjectPayload => "non_object_payload",
            Self::NonObjectProvenance => "non_object_provenance",
            Self::ExpiredCredentialLease => "expired_credential_lease",
            Self::EmptyCredentialLeaseSecret => "empty_credential_lease_secret",
        }
    }
}
